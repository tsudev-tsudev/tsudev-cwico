//! `cwico` - the headless interface to tsudev-cwico.
//!
//! Everything the GUI can do, scriptable. Three reasons it exists:
//!
//! * **Auditing.** `cwico scan --json` gives you the classified inventory of a
//!   machine as data, which is what you want when you are deciding what a
//!   fleet-wide policy should say.
//! * **Automation.** Imaging a hundred laptops should not involve clicking.
//! * **Testing on real hardware.** A GUI is a poor place to find out that a
//!   registry pass has a bug.
//!
//! The safety model is identical to the GUI's, because it is the same engine:
//! `Critical` items are refused, `Caution` items need `--confirm`, and a plan
//! is a dry run unless `--apply` is passed.

use cwico_core::backend::{Event, EventSink, LogLevel, PlatformBackend, StepStatus};
use cwico_core::{
    Action, Engine, ItemFilter, Locale, PlanOptions, RemovalPlan, SafetyClass, SafetyDatabase,
    ScanOptions, ScanReport, Selection, SortBy, SourceKind,
};
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

const HELP: &str = concat!(
    "tsudev-cwico ",
    env!("CARGO_PKG_VERSION"),
    r#" - deep Windows debloater and software removal toolkit
https://tsudev.com

USAGE
    cwico <COMMAND> [OPTIONS]

COMMANDS
    scan                 Inventory the machine and classify everything found
    plan                 Show what removing the selected items would do
    remove               Remove the selected items (dry run unless --apply)
    tweaks               List the system tweaks in the catalogue
    info                 Report the platform, elevation and safety database
    help                 Show this text

SELECTION (plan / remove)
    --id <ID>            Select one item by id. Repeatable.
    --name <TEXT>        Select every item whose name contains TEXT.
    --safe-only          Select every item classified Safe. Never selects
                         Caution, Unknown or Critical items.
    --confirm            Acknowledge Caution and Unknown items. Without this
                         they are reported and skipped.

SCAN OPTIONS
    --quick              Installed programs and AppX packages only
    --deep               Every pass, including residue and disk measurement
    --kind <KIND>        Filter: reg, appx, appxprov, svc, task, startup
    --safety <CLASS>     Filter: safe, caution, unknown, critical
    --search <TEXT>      Filter by name, publisher or identifier

OUTPUT
    --json               Machine-readable output on stdout
    --locale <vi|en>     Language for reasons and descriptions (default: vi)
    --quiet              Errors only

REMOVAL
    --apply              Actually make changes. Everything is a dry run
                         without it.
    --deep-clean         Also sweep leftover folders and registry keys
    --no-restore-point   Skip the System Restore Point. Not recommended:
                         the run then has no single-click rollback.
    --backup-dir <DIR>   Where .reg backups and the transaction log go
                         (default: %LOCALAPPDATA%\tsudev-cwico\backups)

EXAMPLES
    cwico scan --safety safe --locale en
    cwico scan --json > inventory.json
    cwico remove --name OneDrive --deep-clean --apply
    cwico remove --safe-only --apply
"#
);

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Prints progress to stderr so `--json` on stdout stays clean.
struct ConsoleSink {
    quiet: bool,
}

impl EventSink for ConsoleSink {
    fn emit(&self, event: Event) {
        if self.quiet {
            if let Event::Log {
                level: LogLevel::Error,
                message,
            } = &event
            {
                eprintln!("error: {message}");
            }
            return;
        }
        match event {
            Event::ScanPassStarted { pass, index, total } => {
                eprint!("  [{index}/{total}] scanning {pass}… ");
                let _ = std::io::stderr().flush();
            }
            Event::ScanPassFinished { found, .. } => eprintln!("{found} found"),
            Event::ScanFinished { total, duration_ms } => {
                eprintln!("  {total} items in {duration_ms} ms\n");
            }
            Event::RunStarted {
                total_steps,
                dry_run,
            } => {
                eprintln!(
                    "  {} {total_steps} step(s)\n",
                    if dry_run { "Simulating" } else { "Running" }
                );
            }
            Event::StepStarted {
                step, index, total, ..
            } => {
                eprint!("  [{index}/{total}] {step}… ");
                let _ = std::io::stderr().flush();
            }
            Event::StepFinished { status, detail, .. } => {
                let mark = match status {
                    StepStatus::Succeeded => "ok",
                    StepStatus::Skipped => "skip",
                    StepStatus::Simulated => "dry",
                    StepStatus::Failed => "FAIL",
                };
                eprintln!("[{mark}] {detail}");
            }
            Event::ItemFinished { name, status, .. } => {
                eprintln!("  -> {name}: {status:?}\n");
            }
            Event::RunFinished {
                succeeded,
                failed,
                skipped,
                duration_ms,
            } => {
                eprintln!(
                    "\n  {succeeded} succeeded, {failed} failed, {skipped} skipped \
                     in {duration_ms} ms"
                );
            }
            Event::Log { level, message } => eprintln!("  {level:?}: {message}"),
            Event::ScanStarted { .. } => {}
        }
    }
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

fn safety_tag(class: SafetyClass) -> &'static str {
    match class {
        SafetyClass::Safe => "SAFE    ",
        SafetyClass::Caution => "CAUTION ",
        SafetyClass::Unknown => "UNKNOWN ",
        SafetyClass::Critical => "CRITICAL",
    }
}

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

#[derive(Debug, Default)]
struct Args {
    command: String,
    ids: Vec<String>,
    names: Vec<String>,
    safe_only: bool,
    confirm: bool,
    scan: Option<ScanOptions>,
    kinds: Vec<SourceKind>,
    safety: Vec<SafetyClass>,
    search: String,
    json: bool,
    locale: Locale,
    quiet: bool,
    apply: bool,
    deep_clean: bool,
    no_restore_point: bool,
    backup_dir: Option<PathBuf>,
}

fn parse_kind(s: &str) -> Option<SourceKind> {
    Some(match s.to_ascii_lowercase().as_str() {
        "reg" | "registry" | "program" => SourceKind::RegistryUninstall,
        "appx" | "uwp" | "msix" => SourceKind::AppxPackage,
        "appxprov" | "provisioned" => SourceKind::AppxProvisioned,
        "svc" | "service" => SourceKind::WindowsService,
        "task" | "scheduled" => SourceKind::ScheduledTask,
        "startup" | "autostart" => SourceKind::StartupEntry,
        _ => return None,
    })
}

fn parse_safety(s: &str) -> Option<SafetyClass> {
    Some(match s.to_ascii_lowercase().as_str() {
        "safe" => SafetyClass::Safe,
        "caution" => SafetyClass::Caution,
        "unknown" => SafetyClass::Unknown,
        "critical" => SafetyClass::Critical,
        _ => return None,
    })
}

fn parse_args() -> Result<Args, String> {
    let mut raw = std::env::args().skip(1);
    let mut args = Args {
        command: raw.next().unwrap_or_else(|| "help".into()),
        ..Default::default()
    };
    let mut scan = ScanOptions::default();
    let mut scan_touched = false;

    while let Some(flag) = raw.next() {
        let mut value = || raw.next().ok_or_else(|| format!("`{flag}` needs a value"));
        match flag.as_str() {
            "--id" => args.ids.push(value()?),
            "--name" => args.names.push(value()?),
            "--safe-only" => args.safe_only = true,
            "--confirm" => args.confirm = true,
            "--quick" => {
                scan = ScanOptions::quick();
                scan_touched = true;
            }
            "--deep" => {
                scan = ScanOptions::deep();
                scan_touched = true;
            }
            "--kind" => {
                let v = value()?;
                args.kinds
                    .push(parse_kind(&v).ok_or_else(|| format!("unknown kind `{v}`"))?);
            }
            "--safety" => {
                let v = value()?;
                args.safety
                    .push(parse_safety(&v).ok_or_else(|| format!("unknown safety class `{v}`"))?);
            }
            "--search" => args.search = value()?,
            "--json" => args.json = true,
            "--locale" => {
                args.locale = match value()?.to_ascii_lowercase().as_str() {
                    "en" => Locale::En,
                    _ => Locale::Vi,
                }
            }
            "--quiet" => args.quiet = true,
            "--apply" => args.apply = true,
            "--deep-clean" => args.deep_clean = true,
            "--no-restore-point" => args.no_restore_point = true,
            "--backup-dir" => args.backup_dir = Some(PathBuf::from(value()?)),
            "-h" | "--help" => args.command = "help".into(),
            other => return Err(format!("unknown option `{other}`")),
        }
    }

    args.scan = scan_touched.then_some(scan);
    Ok(args)
}

// ---------------------------------------------------------------------------
// Backend selection
// ---------------------------------------------------------------------------

/// The real backend: a Windows target without the `mock` override.
#[cfg(all(windows, not(feature = "mock")))]
fn make_backend() -> Box<dyn PlatformBackend> {
    Box::new(cwico_win::WindowsBackend::new())
}

/// The fixture backend: any non-Windows host, or `--features mock` on Windows.
///
/// A non-Windows build has no Windows to inspect, so this is the only sensible
/// backend there. `platform_info().platform` reports `mock`, and the CLI's
/// `info` command shows it, so it is never mistaken for a real inventory.
#[cfg(any(not(windows), feature = "mock"))]
fn make_backend() -> Box<dyn PlatformBackend> {
    Box::new(cwico_core::mock::MockBackend::new())
}

fn default_backup_dir() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("tsudev-cwico")
        .join("backups")
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/// Does `needle` name this item?
///
/// Deliberately **not** a plain substring test. `--name Edge` matching
/// "Acme L-edge-r Desktop" is the kind of surprise that removes software the
/// user never asked about, so a match must begin at a word boundary: the
/// start of the haystack, or after a character that is not alphanumeric.
///
/// A multi-word needle ("microsoft edge") requires every word to match under
/// the same rule, so it narrows rather than widens.
fn name_matches(haystack_lower: &str, needle: &str) -> bool {
    let words: Vec<String> = needle
        .split_whitespace()
        .map(str::to_lowercase)
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() {
        return false;
    }
    words
        .iter()
        .all(|word| word_boundary_contains(haystack_lower, word))
}

fn word_boundary_contains(haystack_lower: &str, needle_lower: &str) -> bool {
    let mut from = 0usize;
    while let Some(found) = haystack_lower[from..].find(needle_lower) {
        let start = from + found;
        let preceding = haystack_lower[..start].chars().next_back();
        if preceding.is_none_or(|c| !c.is_alphanumeric()) {
            return true;
        }
        from = start + needle_lower.len().max(1);
        if from >= haystack_lower.len() {
            break;
        }
    }
    false
}

fn build_selections(report: &ScanReport, args: &Args) -> Vec<Selection> {
    let action = if args.deep_clean {
        Action::UninstallAndDeepClean
    } else {
        Action::Uninstall
    };

    let mut picked: Vec<&cwico_core::SoftwareItem> = Vec::new();

    for id in &args.ids {
        if let Some(item) = report.get(id) {
            picked.push(item);
        } else {
            eprintln!("warning: no item with id `{id}`");
        }
    }
    for needle in &args.names {
        let matches: Vec<_> = report
            .items
            .iter()
            .filter(|i| name_matches(&i.search_haystack(), needle))
            .collect();
        if matches.is_empty() {
            eprintln!("warning: nothing matched `{needle}`");
        }
        picked.extend(matches);
    }
    if args.safe_only {
        picked.extend(report.bulk_selectable());
    }

    picked.sort_by(|a, b| a.id.cmp(&b.id));
    picked.dedup_by(|a, b| a.id == b.id);

    picked
        .into_iter()
        .map(|item| {
            let selection = Selection::uninstall(&item.id).with_action(action);
            if args.confirm {
                selection.confirmed()
            } else {
                selection
            }
        })
        .collect()
}

fn print_report(report: &ScanReport, args: &Args) {
    let filter = ItemFilter {
        query: args.search.clone(),
        kinds: args.kinds.clone(),
        safety: args.safety.clone(),
        tags: Vec::new(),
        removable_only: false,
    };
    let mut items = filter.apply(&report.items);
    cwico_core::scan::sort_items(&mut items, SortBy::Safety, args.locale);

    println!(
        "\n  {:<8}  {:<9}  {:<46}  {:>10}  VERSION",
        "SAFETY", "KIND", "NAME", "SIZE"
    );
    println!("  {}", "-".repeat(96));

    for item in &items {
        let name = if item.name.chars().count() > 45 {
            let truncated: String = item.name.chars().take(42).collect();
            format!("{truncated}...")
        } else {
            item.name.clone()
        };
        println!(
            "  {}  {:<9}  {:<46}  {:>10}  {}",
            safety_tag(item.safety),
            item.source.slug(),
            name,
            item.size_bytes.map(human_bytes).unwrap_or_default(),
            item.version.clone().unwrap_or_default()
        );
    }

    println!(
        "\n  {} item(s) shown of {}",
        items.len(),
        report.stats.total
    );
    for (class, count) in &report.stats.by_safety {
        print!("  {class}: {count}");
    }
    println!(
        "\n  reclaimable: {}",
        human_bytes(report.stats.reclaimable_bytes)
    );
    if !report.elevated {
        println!(
            "\n  note: not running as Administrator - machine-wide programs and service \
             configuration are incomplete."
        );
    }
    for warning in &report.warnings {
        println!("  warning [{}]: {}", warning.pass, warning.message);
    }
    println!();
}

fn run() -> Result<ExitCode, String> {
    let args = parse_args()?;

    if args.command == "help" || args.command == "--help" {
        print!("{HELP}");
        return Ok(ExitCode::SUCCESS);
    }

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("CWICO_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let backend = make_backend();
    let db = SafetyDatabase::builtin();
    let engine = Engine::new(backend.as_ref(), &db);
    let sink = ConsoleSink {
        quiet: args.quiet || args.json,
    };

    match args.command.as_str() {
        "info" => {
            let info = backend.platform_info();
            let (safe, caution, critical) = db.class_counts();
            if args.json {
                let value = serde_json::json!({
                    "platform": info,
                    "safetyDatabase": {
                        "version": db.version(),
                        "updated": db.updated(),
                        "rules": db.rules().len(),
                        "safe": safe, "caution": caution, "critical": critical,
                    },
                    "version": env!("CARGO_PKG_VERSION"),
                });
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
            } else {
                println!("\n  tsudev-cwico {}", env!("CARGO_PKG_VERSION"));
                println!("  https://tsudev.com\n");
                println!("  platform         {}", info.platform);
                println!("  os               {}", info.os_description);
                println!("  architecture     {}", info.arch);
                println!(
                    "  elevated         {}",
                    if info.elevated { "yes" } else { "no" }
                );
                println!(
                    "  system restore   {}",
                    if info.system_restore_available {
                        "available"
                    } else {
                        "UNAVAILABLE - enable System Protection before removing anything"
                    }
                );
                println!(
                    "\n  safety database  v{} ({}), {} rules",
                    db.version(),
                    db.updated(),
                    db.rules().len()
                );
                println!("    {safe} safe, {caution} caution, {critical} critical\n");
            }
            Ok(ExitCode::SUCCESS)
        }

        "tweaks" => {
            let catalog = cwico_core::TweakCatalog::builtin();
            if args.json {
                println!("{}", serde_json::to_string_pretty(&catalog).unwrap());
            } else {
                println!("\n  {} tweaks\n", catalog.tweaks.len());
                for tweak in &catalog.tweaks {
                    println!(
                        "  {}  {:<34}  {}",
                        safety_tag(tweak.safety),
                        tweak.id,
                        tweak.title.get(args.locale)
                    );
                    println!("            {}", tweak.description.get(args.locale));
                    if !tweak.is_reversible() {
                        println!("            [one-way: this tweak has no revert path]");
                    }
                    println!();
                }
            }
            Ok(ExitCode::SUCCESS)
        }

        "scan" => {
            let options = args.scan.clone().unwrap_or_default();
            let report = engine.scan(&options, &sink).map_err(|e| e.to_string())?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                print_report(&report, &args);
            }
            Ok(ExitCode::SUCCESS)
        }

        command @ ("plan" | "remove") => {
            let options = args.scan.clone().unwrap_or_default();
            let report = engine.scan(&options, &sink).map_err(|e| e.to_string())?;
            let selections = build_selections(&report, &args);

            if selections.is_empty() {
                eprintln!("nothing selected. Use --id, --name or --safe-only.");
                return Ok(ExitCode::from(2));
            }

            let plan_options = PlanOptions {
                dry_run: command == "plan" || !args.apply,
                create_restore_point: !args.no_restore_point,
                require_restore_point: !args.no_restore_point,
                backup_dir: Some(args.backup_dir.clone().unwrap_or_else(default_backup_dir)),
                ..PlanOptions::default()
            };
            let plan = RemovalPlan::build(&report, &selections, plan_options);

            for rejection in &plan.rejected {
                eprintln!(
                    "  skipped {}: [{}] {}",
                    rejection.name, rejection.code, rejection.detail
                );
            }
            if !plan.rejected.is_empty() {
                eprintln!();
            }

            if plan.is_empty() {
                eprintln!("nothing left to do after safety checks.");
                return Ok(ExitCode::from(2));
            }

            // Say plainly what is about to happen. A destructive command must
            // never surprise the user with an item they did not mean to name.
            if command == "remove" {
                eprintln!(
                    "  {} item(s) selected{}:",
                    plan.items.len(),
                    if args.apply { "" } else { " (dry run)" }
                );
                for item in &plan.items {
                    eprintln!("    {} {}", safety_tag(item.safety), item.name);
                }
                eprintln!();
            }

            if command == "plan" {
                if args.json {
                    println!("{}", serde_json::to_string_pretty(&plan).unwrap());
                } else {
                    println!(
                        "\n  {} item(s), {} step(s):\n",
                        plan.items.len(),
                        plan.total_steps()
                    );
                    for item in &plan.items {
                        println!("  {}  {}", safety_tag(item.safety), item.name);
                        for step in &item.steps {
                            println!("      - {}", step.slug());
                        }
                    }
                    println!("\n  Add --apply to carry this out.\n");
                }
                return Ok(ExitCode::SUCCESS);
            }

            if !args.apply {
                eprintln!("  (dry run - add --apply to make changes)\n");
            }

            let run_report = engine.execute(&plan, &sink).map_err(|e| e.to_string())?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&run_report).unwrap());
            } else {
                println!("\n  {} freed", human_bytes(run_report.bytes_freed));
                if let Some(point) = &run_report.restore_point {
                    println!("  restore point #{}", point.sequence_number);
                }
                if let Some(log) = &run_report.transaction_log {
                    println!("  transaction log: {}", log.display());
                }
                if run_report.reboot_required {
                    println!("\n  A restart is required to finish.");
                }
                println!();
            }

            Ok(if run_report.all_succeeded() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            })
        }

        other => Err(format!(
            "unknown command `{other}`. Run `cwico help` for usage."
        )),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_needle_matches_at_a_word_boundary() {
        assert!(name_matches("microsoft edge", "edge"));
        assert!(name_matches("microsoft edge", "Edge"));
        assert!(name_matches("microsoft edge update", "microsoft"));
    }

    #[test]
    fn a_needle_does_not_match_inside_a_longer_word() {
        // The bug this rule exists to prevent: `--name Edge` selecting
        // "Acme Ledger Desktop" and removing a line-of-business application.
        assert!(!name_matches("acme ledger desktop", "edge"));
        assert!(!name_matches("windows subsystem for linux", "ux"));
        assert!(!name_matches("microsoft onedrive", "rive"));
    }

    #[test]
    fn a_prefix_of_a_word_still_matches() {
        // Users type partial names; matching a word's start is expected.
        assert!(name_matches("microsoft onedrive", "one"));
        assert!(name_matches("candy crush saga", "candy"));
    }

    #[test]
    fn a_needle_after_punctuation_matches() {
        assert!(name_matches("king.com.candycrushsaga", "com"));
        assert!(name_matches("microsoft-edge-webview", "edge"));
    }

    #[test]
    fn every_word_of_a_multi_word_needle_must_match() {
        assert!(name_matches("microsoft edge update", "microsoft edge"));
        assert!(!name_matches(
            "microsoft edge update",
            "microsoft photoshop"
        ));
    }

    #[test]
    fn an_empty_needle_matches_nothing() {
        assert!(!name_matches("anything at all", ""));
        assert!(!name_matches("anything at all", "   "));
    }

    #[test]
    fn kinds_and_safety_classes_parse_from_their_cli_spellings() {
        assert_eq!(parse_kind("appx"), Some(SourceKind::AppxPackage));
        assert_eq!(parse_kind("svc"), Some(SourceKind::WindowsService));
        assert_eq!(parse_kind("nonsense"), None);
        assert_eq!(parse_safety("critical"), Some(SafetyClass::Critical));
        assert_eq!(parse_safety("nonsense"), None);
    }

    #[test]
    fn byte_sizes_are_human_readable() {
        assert_eq!(human_bytes(512), "512 B");
        assert_eq!(human_bytes(1_536), "1.5 KB");
        assert_eq!(human_bytes(1_932_735_283), "1.8 GB");
    }
}
