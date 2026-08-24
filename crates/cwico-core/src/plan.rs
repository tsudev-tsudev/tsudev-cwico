//! Turning a user's selection into an ordered, auditable plan.
//!
//! Nothing in this crate touches the system directly. The plan is built and
//! validated first - including the hard block on `Critical` items - and only
//! then handed to a [`crate::backend::PlatformBackend`] for execution. That
//! split is what makes the dangerous half of the tool testable on any host.

use crate::error::{Error, Result};
use crate::model::{Action, SafetyClass, SoftwareItem, SourceKind};
use crate::scan::ScanReport;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Request
// ---------------------------------------------------------------------------

/// One line of the user's selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub item_id: String,
    pub action: Action,
    /// Set by the UI when the user has acknowledged a `Caution`/`Unknown`
    /// warning for this specific item. Bulk actions cannot set it.
    #[serde(default)]
    pub confirmed: bool,
}

impl Selection {
    pub fn uninstall(item_id: impl Into<String>) -> Self {
        Self {
            item_id: item_id.into(),
            action: Action::Uninstall,
            confirmed: false,
        }
    }

    pub fn confirmed(mut self) -> Self {
        self.confirmed = true;
        self
    }

    pub fn with_action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }
}

/// Run-wide switches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanOptions {
    /// Create a System Restore Point before the first destructive step.
    pub create_restore_point: bool,
    /// Abort the whole run if the restore point cannot be created.
    /// On by default: a rollback you cannot perform is not a rollback.
    pub require_restore_point: bool,
    /// Export every registry key the run will touch to a `.reg` file first.
    pub backup_registry: bool,
    /// Terminate the item's processes before invoking its uninstaller.
    pub kill_processes: bool,
    /// Also deprovision AppX packages so they do not return for new users.
    pub remove_provisioned: bool,
    /// Walk the plan without changing anything. Every step reports what it
    /// *would* do. This is the default for a first run in the UI.
    pub dry_run: bool,
    /// Keep going when one item fails, instead of stopping the run.
    pub continue_on_error: bool,
    /// Where restore artefacts (`.reg` exports, the transaction log) are written.
    pub backup_dir: Option<PathBuf>,
}

impl Default for PlanOptions {
    fn default() -> Self {
        Self {
            create_restore_point: true,
            require_restore_point: true,
            backup_registry: true,
            kill_processes: true,
            remove_provisioned: true,
            dry_run: false,
            continue_on_error: true,
            backup_dir: None,
        }
    }
}

impl PlanOptions {
    /// Preview mode: everything is simulated, no safety scaffolding needed.
    pub fn dry_run() -> Self {
        Self {
            dry_run: true,
            create_restore_point: false,
            require_restore_point: false,
            backup_registry: false,
            ..Self::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Steps
// ---------------------------------------------------------------------------

/// One executable unit of work. The order of this enum's variants in a
/// [`PlannedItem`] is the order they run in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Step {
    /// Snapshot the machine with `SRSetRestorePointW`. Run-level, once.
    CreateRestorePoint { description: String },
    /// Export registry keys to `.reg` files before touching them.
    BackupRegistry { keys: Vec<String> },
    /// Terminate every running process belonging to the item.
    KillProcesses { executables: Vec<String> },
    /// Stop and disable services the item owns.
    StopServices { services: Vec<String> },
    /// Disable scheduled tasks the item owns.
    DisableTasks { tasks: Vec<String> },
    /// Invoke the vendor's own uninstaller, silently when possible.
    RunOfficialUninstaller { command: String, silent: bool },
    /// `Remove-AppxPackage -AllUsers` equivalent, via the packaging API.
    RemoveAppxPackage {
        package_full_name: String,
        all_users: bool,
    },
    /// Deprovision so the package does not come back for new user accounts.
    RemoveAppxProvisioned { package_name: String },
    /// Delete residual directories.
    DeepCleanFiles { paths: Vec<String> },
    /// Delete residual registry keys and values.
    DeepCleanRegistry {
        keys: Vec<String>,
        values: Vec<String>,
    },
    /// Reversibly disable a service (`StartType = Disabled`).
    SetServiceStartup {
        service: String,
        /// `false` disables, `true` restores automatic start.
        enabled: bool,
    },
    /// Reversibly enable/disable a scheduled task.
    SetTaskEnabled { task: String, enabled: bool },
    /// Remove a `Run` value or Startup-folder shortcut.
    RemoveStartupEntry { location: String, name: String },
}

impl Step {
    /// Stable identifier used by the UI to label the step and by the log.
    pub fn slug(&self) -> &'static str {
        match self {
            Step::CreateRestorePoint { .. } => "create_restore_point",
            Step::BackupRegistry { .. } => "backup_registry",
            Step::KillProcesses { .. } => "kill_processes",
            Step::StopServices { .. } => "stop_services",
            Step::DisableTasks { .. } => "disable_tasks",
            Step::RunOfficialUninstaller { .. } => "run_official_uninstaller",
            Step::RemoveAppxPackage { .. } => "remove_appx_package",
            Step::RemoveAppxProvisioned { .. } => "remove_appx_provisioned",
            Step::DeepCleanFiles { .. } => "deep_clean_files",
            Step::DeepCleanRegistry { .. } => "deep_clean_registry",
            Step::SetServiceStartup { .. } => "set_service_startup",
            Step::SetTaskEnabled { .. } => "set_task_enabled",
            Step::RemoveStartupEntry { .. } => "remove_startup_entry",
        }
    }

    /// `true` when the step changes the system in a way a restore point is
    /// meant to cover.
    pub fn is_destructive(&self) -> bool {
        !matches!(
            self,
            Step::CreateRestorePoint { .. } | Step::BackupRegistry { .. }
        )
    }
}

/// One item's worth of work.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedItem {
    pub item_id: String,
    pub name: String,
    pub source: SourceKind,
    pub safety: SafetyClass,
    pub action: Action,
    pub steps: Vec<Step>,
    /// Registry keys this item's steps will read or delete, collected so the
    /// run-level backup can export them in one pass.
    pub registry_keys_at_risk: Vec<String>,
}

/// A validated, ordered plan. Constructing one is the safety gate: a
/// `RemovalPlan` cannot contain a `Critical` item, by construction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovalPlan {
    pub options: PlanOptions,
    /// Run-level preamble: restore point, then the consolidated registry backup.
    pub preamble: Vec<Step>,
    pub items: Vec<PlannedItem>,
    /// Selections that were dropped, with the reason. Surfaced in the UI so a
    /// silently-skipped item is never a surprise.
    pub rejected: Vec<RejectedSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectedSelection {
    pub item_id: String,
    pub name: String,
    /// Machine-readable: `protected_component`, `needs_confirmation`,
    /// `not_removable`, `unknown_item`.
    pub code: String,
    pub detail: String,
}

impl RemovalPlan {
    /// Build and validate a plan.
    ///
    /// Rejections rather than errors: one bad selection must not invalidate
    /// the other 40. The caller shows [`RemovalPlan::rejected`] to the user.
    pub fn build(report: &ScanReport, selections: &[Selection], options: PlanOptions) -> Self {
        let mut items = Vec::new();
        let mut rejected = Vec::new();
        let mut all_registry_keys: Vec<String> = Vec::new();

        for sel in selections {
            let Some(item) = report.get(&sel.item_id) else {
                rejected.push(RejectedSelection {
                    item_id: sel.item_id.clone(),
                    name: sel.item_id.clone(),
                    code: "unknown_item".into(),
                    detail: "not present in the current scan; rescan and try again".into(),
                });
                continue;
            };

            // --- The hard block. Nothing overrides this. --------------------
            if item.safety.is_blocked() {
                rejected.push(RejectedSelection {
                    item_id: item.id.clone(),
                    name: item.name.clone(),
                    code: "protected_component".into(),
                    detail: item
                        .safety_reason
                        .as_ref()
                        .map(|r| r.en.clone())
                        .unwrap_or_else(|| "classified Critical by the safety database".into()),
                });
                continue;
            }

            // --- Caution/Unknown need an explicit per-item acknowledgement --
            if item.safety.needs_confirmation() && !sel.confirmed && sel.action.is_destructive() {
                rejected.push(RejectedSelection {
                    item_id: item.id.clone(),
                    name: item.name.clone(),
                    code: "needs_confirmation".into(),
                    detail: "this item is Caution or Unknown and needs an explicit confirmation"
                        .into(),
                });
                continue;
            }

            // --- Can we actually act on it? ---------------------------------
            let actionable = match sel.action {
                Action::Disable | Action::Enable => item.can_disable,
                _ => item.can_uninstall,
            };
            if !actionable {
                rejected.push(RejectedSelection {
                    item_id: item.id.clone(),
                    name: item.name.clone(),
                    code: "not_removable".into(),
                    detail: format!(
                        "no {} path is available for this item",
                        if matches!(sel.action, Action::Disable | Action::Enable) {
                            "disable"
                        } else {
                            "uninstall"
                        }
                    ),
                });
                continue;
            }

            let planned = Self::plan_item(item, sel.action, &options);
            all_registry_keys.extend(planned.registry_keys_at_risk.iter().cloned());
            items.push(planned);
        }

        // Deterministic order: services and tasks are quiesced before the
        // programs that own them, and AppX removal precedes deep cleaning.
        items.sort_by_key(|p| Self::execution_rank(p.source, p.action));

        let mut preamble = Vec::new();
        let has_destructive = items
            .iter()
            .any(|p| p.steps.iter().any(Step::is_destructive));

        if options.create_restore_point && has_destructive && !options.dry_run {
            preamble.push(Step::CreateRestorePoint {
                description: format!(
                    "tsudev-cwico: before removing {} item{}",
                    items.len(),
                    if items.len() == 1 { "" } else { "s" }
                ),
            });
        }
        if options.backup_registry && !all_registry_keys.is_empty() && !options.dry_run {
            all_registry_keys.sort();
            all_registry_keys.dedup();
            preamble.push(Step::BackupRegistry {
                keys: all_registry_keys,
            });
        }

        Self {
            options,
            preamble,
            items,
            rejected,
        }
    }

    /// Lower runs earlier.
    fn execution_rank(source: SourceKind, action: Action) -> u8 {
        match (source, action) {
            (_, Action::Enable) => 0,
            (SourceKind::ScheduledTask, _) => 1,
            (SourceKind::StartupEntry, _) => 2,
            (SourceKind::WindowsService, _) => 3,
            (SourceKind::AppxPackage, _) => 4,
            (SourceKind::AppxProvisioned, _) => 5,
            (SourceKind::RegistryUninstall, _) => 6,
            (SourceKind::OptionalFeature | SourceKind::WindowsCapability, _) => 7,
            (SourceKind::Leftover, _) => 8,
        }
    }

    fn plan_item(item: &SoftwareItem, action: Action, options: &PlanOptions) -> PlannedItem {
        let mut steps = Vec::new();
        let mut registry_keys_at_risk = Vec::new();

        if let Some(key) = &item.registry_key {
            registry_keys_at_risk.push(key.clone());
        }

        match action {
            // ---- Reversible state changes -------------------------------
            Action::Disable | Action::Enable => {
                let enabled = matches!(action, Action::Enable);
                let name = item
                    .system_name
                    .clone()
                    .unwrap_or_else(|| item.name.clone());
                match item.source {
                    SourceKind::WindowsService => {
                        if !enabled && options.kill_processes {
                            steps.push(Step::StopServices {
                                services: vec![name.clone()],
                            });
                        }
                        steps.push(Step::SetServiceStartup {
                            service: name,
                            enabled,
                        });
                    }
                    SourceKind::ScheduledTask => {
                        steps.push(Step::SetTaskEnabled {
                            task: name,
                            enabled,
                        });
                    }
                    SourceKind::StartupEntry => {
                        let location = item
                            .extra
                            .get("startupLocation")
                            .cloned()
                            .unwrap_or_default();
                        if location.starts_with("HK") {
                            registry_keys_at_risk.push(location.clone());
                        }
                        steps.push(Step::RemoveStartupEntry {
                            location,
                            name: item.name.clone(),
                        });
                    }
                    _ => {}
                }
            }

            // ---- Residue sweep only -------------------------------------
            Action::DeepCleanOnly => {
                if Self::leaves_residue(item.source) {
                    Self::push_deep_clean(item, &mut steps, &mut registry_keys_at_risk);
                }
            }

            // ---- Full removal -------------------------------------------
            Action::Uninstall | Action::UninstallAndDeepClean => {
                // 1. Quiesce anything that would hold files open.
                if options.kill_processes && !item.executables.is_empty() {
                    steps.push(Step::KillProcesses {
                        executables: item.executables.clone(),
                    });
                }
                if let Some(services) = item.extra.get("relatedServices") {
                    let list = split_list(services);
                    if !list.is_empty() {
                        steps.push(Step::StopServices { services: list });
                    }
                }
                if let Some(tasks) = item.extra.get("relatedTasks") {
                    let list = split_list(tasks);
                    if !list.is_empty() {
                        steps.push(Step::DisableTasks { tasks: list });
                    }
                }

                // 2. The vendor's own uninstaller, or the packaging API.
                match item.source {
                    SourceKind::AppxPackage | SourceKind::AppxProvisioned => {
                        if let Some(full_name) = &item.package_full_name {
                            steps.push(Step::RemoveAppxPackage {
                                package_full_name: full_name.clone(),
                                all_users: true,
                            });
                        }
                        if options.remove_provisioned {
                            if let Some(name) = item
                                .extra
                                .get("provisionedPackageName")
                                .or(item.package_full_name.as_ref())
                            {
                                steps.push(Step::RemoveAppxProvisioned {
                                    package_name: name.clone(),
                                });
                            }
                        }
                    }
                    SourceKind::WindowsService => {
                        let name = item
                            .system_name
                            .clone()
                            .unwrap_or_else(|| item.name.clone());
                        steps.push(Step::StopServices {
                            services: vec![name.clone()],
                        });
                        steps.push(Step::SetServiceStartup {
                            service: name,
                            enabled: false,
                        });
                    }
                    SourceKind::ScheduledTask => {
                        let name = item
                            .system_name
                            .clone()
                            .unwrap_or_else(|| item.name.clone());
                        steps.push(Step::SetTaskEnabled {
                            task: name,
                            enabled: false,
                        });
                    }
                    SourceKind::StartupEntry => {
                        let location = item
                            .extra
                            .get("startupLocation")
                            .cloned()
                            .unwrap_or_default();
                        steps.push(Step::RemoveStartupEntry {
                            location,
                            name: item.name.clone(),
                        });
                    }
                    _ => {
                        if let Some(cmd) = item.preferred_uninstall_command() {
                            steps.push(Step::RunOfficialUninstaller {
                                command: cmd.to_string(),
                                silent: item.has_native_silent_uninstall(),
                            });
                        }
                    }
                }

                // 3. Residue - but only for kinds that leave any.
                //
                // A service or a scheduled task is *disabled*, never deleted,
                // and its "registry key" is its definition in the service
                // control database. Sweeping that would delete the service
                // outright - the opposite of the reversible change the user
                // asked for, and unrecoverable without a reinstall.
                if matches!(action, Action::UninstallAndDeepClean)
                    && Self::leaves_residue(item.source)
                {
                    Self::push_deep_clean(item, &mut steps, &mut registry_keys_at_risk);
                }
            }
        }

        PlannedItem {
            item_id: item.id.clone(),
            name: item.name.clone(),
            source: item.source,
            safety: item.safety,
            action,
            steps,
            registry_keys_at_risk,
        }
    }

    /// Whether removing this kind of item can leave residue worth sweeping.
    ///
    /// Programs and packages install files and write registry trees, so they
    /// do. Services, scheduled tasks and autostart entries *are* the registry
    /// entry - turning them off is the whole operation, and deleting their
    /// key would destroy something the user expects to be able to turn back on.
    fn leaves_residue(source: SourceKind) -> bool {
        matches!(
            source,
            SourceKind::RegistryUninstall
                | SourceKind::AppxPackage
                | SourceKind::AppxProvisioned
                | SourceKind::Leftover
        )
    }

    fn push_deep_clean(item: &SoftwareItem, steps: &mut Vec<Step>, at_risk: &mut Vec<String>) {
        let mut paths: Vec<String> = item
            .extra
            .get("leftoverPaths")
            .map(|s| split_list(s))
            .unwrap_or_default();
        if let Some(loc) = &item.install_location {
            paths.push(loc.to_string_lossy().into_owned());
        }
        if let Some(family) = &item.package_family_name {
            paths.push(format!("%LOCALAPPDATA%\\Packages\\{family}"));
        }
        paths.sort();
        paths.dedup();
        if !paths.is_empty() {
            steps.push(Step::DeepCleanFiles { paths });
        }

        let mut keys: Vec<String> = item
            .extra
            .get("leftoverRegistry")
            .map(|s| split_list(s))
            .unwrap_or_default();
        if let Some(key) = &item.registry_key {
            keys.push(key.clone());
        }
        let values: Vec<String> = item
            .extra
            .get("leftoverRegistryValues")
            .map(|s| split_list(s))
            .unwrap_or_default();

        keys.sort();
        keys.dedup();
        at_risk.extend(keys.iter().cloned());
        at_risk.extend(
            values
                .iter()
                .filter_map(|v| v.split_once("::").map(|(key, _value)| key.to_string())),
        );

        if !keys.is_empty() || !values.is_empty() {
            steps.push(Step::DeepCleanRegistry { keys, values });
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn total_steps(&self) -> usize {
        self.preamble.len() + self.items.iter().map(|i| i.steps.len()).sum::<usize>()
    }

    /// Assert the invariant the type is supposed to guarantee. Called by the
    /// engine immediately before execution as a belt-and-braces check: if a
    /// future refactor ever lets a `Critical` item through, the run aborts
    /// rather than proceeding.
    pub fn assert_no_protected_items(&self) -> Result<()> {
        if let Some(bad) = self.items.iter().find(|p| p.safety.is_blocked()) {
            return Err(Error::ProtectedComponent {
                name: bad.name.clone(),
                reason: "a Critical item reached execution; this is a bug in plan construction"
                    .into(),
            });
        }
        Ok(())
    }
}

/// The engine stores multi-valued hints in `extra` as newline-separated lists.
fn split_list(s: &str) -> Vec<String> {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// Serialise a list back into the `extra` representation.
pub fn join_list<I, S>(items: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    items
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::ScanOptions;

    fn report_with(items: Vec<SoftwareItem>) -> ScanReport {
        let mut r = ScanReport::new(ScanOptions::default(), "now".into(), "test".into());
        r.items = items;
        r
    }

    fn safe_program(id: &str, name: &str) -> SoftwareItem {
        let mut i = SoftwareItem::new(id, name, SourceKind::RegistryUninstall);
        i.safety = SafetyClass::Safe;
        i.uninstall_string = Some("C:\\App\\uninst.exe".into());
        i.quiet_uninstall_string = Some("C:\\App\\uninst.exe /S".into());
        i.registry_key = Some("HKLM\\SOFTWARE\\...\\Uninstall\\App".into());
        i.executables = vec!["app.exe".into()];
        i
    }

    #[test]
    fn critical_items_are_rejected_even_when_confirmed() {
        let mut crit = SoftwareItem::new(
            "svc:WinDefend",
            "Windows Defender",
            SourceKind::WindowsService,
        );
        crit.safety = SafetyClass::Critical;
        let report = report_with(vec![crit]);

        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("svc:WinDefend").confirmed()],
            PlanOptions::default(),
        );

        assert!(
            plan.items.is_empty(),
            "a Critical item must never be planned"
        );
        assert_eq!(plan.rejected.len(), 1);
        assert_eq!(plan.rejected[0].code, "protected_component");
        plan.assert_no_protected_items().unwrap();
    }

    #[test]
    fn caution_requires_confirmation() {
        let mut caution = safe_program("reg:edge", "Microsoft Edge");
        caution.safety = SafetyClass::Caution;
        let report = report_with(vec![caution]);

        let unconfirmed = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:edge")],
            PlanOptions::default(),
        );
        assert_eq!(unconfirmed.rejected[0].code, "needs_confirmation");
        assert!(unconfirmed.items.is_empty());

        let confirmed = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:edge").confirmed()],
            PlanOptions::default(),
        );
        assert_eq!(confirmed.items.len(), 1);
        assert!(confirmed.rejected.is_empty());
    }

    #[test]
    fn safe_items_need_no_confirmation() {
        let report = report_with(vec![safe_program("reg:app", "App")]);
        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:app")],
            PlanOptions::default(),
        );
        assert_eq!(plan.items.len(), 1);
    }

    #[test]
    fn uninstall_flow_is_kill_then_uninstall_then_clean() {
        let mut item = safe_program("reg:app", "App");
        item.extra
            .insert("leftoverPaths".into(), join_list(["%LOCALAPPDATA%\\App"]));
        let report = report_with(vec![item]);

        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:app").with_action(Action::UninstallAndDeepClean)],
            PlanOptions::default(),
        );

        let slugs: Vec<&str> = plan.items[0].steps.iter().map(Step::slug).collect();
        assert_eq!(
            slugs,
            vec![
                "kill_processes",
                "run_official_uninstaller",
                "deep_clean_files",
                "deep_clean_registry"
            ]
        );
    }

    #[test]
    fn silent_uninstall_is_preferred() {
        let report = report_with(vec![safe_program("reg:app", "App")]);
        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:app")],
            PlanOptions::default(),
        );
        match &plan.items[0].steps[1] {
            Step::RunOfficialUninstaller { command, silent } => {
                assert!(command.ends_with("/S"));
                assert!(*silent);
            }
            other => panic!("expected the uninstaller step, got {other:?}"),
        }
    }

    #[test]
    fn preamble_creates_restore_point_and_backs_up_registry() {
        let report = report_with(vec![safe_program("reg:app", "App")]);
        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:app")],
            PlanOptions::default(),
        );
        let slugs: Vec<&str> = plan.preamble.iter().map(Step::slug).collect();
        assert_eq!(slugs, vec!["create_restore_point", "backup_registry"]);
    }

    #[test]
    fn dry_run_has_no_preamble() {
        let report = report_with(vec![safe_program("reg:app", "App")]);
        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:app")],
            PlanOptions::dry_run(),
        );
        assert!(plan.preamble.is_empty());
        assert_eq!(plan.items.len(), 1, "dry run still plans the work");
    }

    #[test]
    fn appx_removal_also_deprovisions() {
        let mut appx = SoftwareItem::new("appx:X", "X", SourceKind::AppxPackage);
        appx.safety = SafetyClass::Safe;
        appx.package_full_name = Some("X_1.0.0.0_x64__abc".into());
        let report = report_with(vec![appx]);

        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("appx:X")],
            PlanOptions::default(),
        );
        let slugs: Vec<&str> = plan.items[0].steps.iter().map(Step::slug).collect();
        assert_eq!(
            slugs,
            vec!["remove_appx_package", "remove_appx_provisioned"]
        );
    }

    #[test]
    fn services_are_quiesced_before_the_programs_that_own_them() {
        let mut svc = SoftwareItem::new("svc:AppSvc", "AppSvc", SourceKind::WindowsService);
        svc.safety = SafetyClass::Safe;
        svc.system_name = Some("AppSvc".into());
        let report = report_with(vec![safe_program("reg:app", "App"), svc]);

        let plan = RemovalPlan::build(
            &report,
            &[
                Selection::uninstall("reg:app"),
                Selection::uninstall("svc:AppSvc"),
            ],
            PlanOptions::default(),
        );
        assert_eq!(plan.items[0].source, SourceKind::WindowsService);
        assert_eq!(plan.items[1].source, SourceKind::RegistryUninstall);
    }

    #[test]
    fn deep_clean_never_deletes_a_service_definition() {
        // `HKLM\SYSTEM\CurrentControlSet\Services\<name>` *is* the service.
        // Sweeping it as "residue" would uninstall the service outright,
        // which no amount of re-enabling brings back.
        let mut svc = SoftwareItem::new("svc:Fax", "Fax", SourceKind::WindowsService);
        svc.safety = SafetyClass::Safe;
        svc.system_name = Some("Fax".into());
        svc.registry_key = Some(r"HKLM\SYSTEM\CurrentControlSet\Services\Fax".into());
        let report = report_with(vec![svc]);

        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("svc:Fax").with_action(Action::UninstallAndDeepClean)],
            PlanOptions::default(),
        );

        let slugs: Vec<&str> = plan.items[0].steps.iter().map(Step::slug).collect();
        assert_eq!(slugs, vec!["stop_services", "set_service_startup"]);
        assert!(
            !slugs.iter().any(|s| s.starts_with("deep_clean")),
            "a service must never be deep cleaned: {slugs:?}"
        );
    }

    #[test]
    fn deep_clean_never_touches_scheduled_tasks_or_startup_entries() {
        for (id, name, kind) in [
            ("task:\\X", "X", SourceKind::ScheduledTask),
            ("startup:HKCU\\Run:Y", "Y", SourceKind::StartupEntry),
        ] {
            let mut item = SoftwareItem::new(id, name, kind);
            item.safety = SafetyClass::Safe;
            item.system_name = Some(name.into());
            item.registry_key = Some(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".into());
            let report = report_with(vec![item]);

            let plan = RemovalPlan::build(
                &report,
                &[Selection::uninstall(id).with_action(Action::UninstallAndDeepClean)],
                PlanOptions::default(),
            );
            let slugs: Vec<&str> = plan.items[0].steps.iter().map(Step::slug).collect();
            assert!(
                !slugs.iter().any(|s| s.starts_with("deep_clean")),
                "`{name}` ({kind:?}) must not be deep cleaned: {slugs:?}"
            );
        }
    }

    #[test]
    fn programs_and_packages_still_get_their_residue_swept() {
        let mut item = safe_program("reg:app", "App");
        item.extra
            .insert("leftoverPaths".into(), join_list([r"C:\Vendor\App"]));
        let report = report_with(vec![item]);

        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:app").with_action(Action::UninstallAndDeepClean)],
            PlanOptions::default(),
        );
        let slugs: Vec<&str> = plan.items[0].steps.iter().map(Step::slug).collect();
        assert!(slugs.contains(&"deep_clean_files"));
        assert!(slugs.contains(&"deep_clean_registry"));
    }

    #[test]
    fn unknown_ids_are_reported_not_silently_dropped() {
        let report = report_with(vec![]);
        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("reg:ghost")],
            PlanOptions::default(),
        );
        assert_eq!(plan.rejected[0].code, "unknown_item");
    }

    #[test]
    fn disable_action_on_a_service_stops_then_disables() {
        let mut svc = SoftwareItem::new("svc:Fax", "Fax", SourceKind::WindowsService);
        svc.safety = SafetyClass::Safe;
        svc.system_name = Some("Fax".into());
        let report = report_with(vec![svc]);

        let plan = RemovalPlan::build(
            &report,
            &[Selection::uninstall("svc:Fax").with_action(Action::Disable)],
            PlanOptions::default(),
        );
        let slugs: Vec<&str> = plan.items[0].steps.iter().map(Step::slug).collect();
        assert_eq!(slugs, vec!["stop_services", "set_service_startup"]);
    }
}
