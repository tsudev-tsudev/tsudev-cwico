//! Applying and reverting system tweaks.
//!
//! [`cwico_core::tweaks`] describes *what* a tweak changes; this module makes
//! the change. Each [`TweakEffect`] variant maps to exactly one narrow
//! operation, and [`TweakEffect::RunCommand`] is restricted to an allow-list
//! rather than being a general shell escape — a tweak catalogue is data, and
//! data that can run arbitrary commands is a remote code execution bug
//! waiting for someone to ship a malicious catalogue update.

use crate::registry::{RegKey, RegValue, RegView};
use crate::services::{self, StartType};
use cwico_core::backend::StepResult;
use cwico_core::tweaks::{RegValueKind, Tweak, TweakEffect};
use cwico_core::{Error, Result};
use std::process::Command;

/// Programs a tweak may invoke. Anything else is refused.
const ALLOWED_PROGRAMS: &[&str] = &["powercfg", "dism", "tzutil", "netsh"];

/// The internal pseudo-command prefix for actions implemented in Rust rather
/// than by shelling out.
const INTERNAL_PREFIX: &str = "cwico:";

/// Convert a catalogue value string into a typed registry value.
fn to_reg_value(kind: RegValueKind, raw: &str) -> Result<RegValue> {
    let bad = |what: &str| {
        Err(Error::other(format!(
            "tweak value `{raw}` is not a valid {what}"
        )))
    };
    match kind {
        RegValueKind::Dword => raw
            .trim()
            .parse::<u32>()
            .map(RegValue::Dword)
            .or_else(|_| bad("DWORD")),
        RegValueKind::Qword => raw
            .trim()
            .parse::<u64>()
            .map(RegValue::Qword)
            .or_else(|_| bad("QWORD")),
        RegValueKind::String => Ok(RegValue::Str(raw.to_string())),
        RegValueKind::ExpandString => Ok(RegValue::ExpandStr(raw.to_string())),
        RegValueKind::MultiString => Ok(RegValue::MultiStr(
            raw.lines().map(str::to_string).collect(),
        )),
        RegValueKind::Binary => {
            let bytes: std::result::Result<Vec<u8>, _> = raw
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter(|s| !s.is_empty())
                .map(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16))
                .collect();
            bytes.map(RegValue::Binary).or_else(|_| bad("binary blob"))
        }
    }
}

/// Apply one effect.
pub fn apply_effect(effect: &TweakEffect, dry_run: bool) -> Result<StepResult> {
    match effect {
        TweakEffect::SetRegistryValue {
            key,
            name,
            kind,
            value,
        } => {
            let typed = to_reg_value(*kind, value)?;
            if dry_run {
                return Ok(StepResult::simulated(format!(
                    "would set {key}::{name} = {value}"
                )));
            }
            // Policy keys usually do not exist yet, so create rather than open.
            let reg_key = RegKey::create_path(key, RegView::Bits64)?;
            reg_key.set_value(name, &typed)?;
            Ok(StepResult::ok(format!("{key}::{name} = {value}")))
        }

        TweakEffect::DeleteRegistryValue { key, name } => {
            if dry_run {
                return Ok(StepResult::simulated(format!("would delete {key}::{name}")));
            }
            match RegKey::open_path(
                key,
                RegView::Bits64,
                windows::Win32::System::Registry::KEY_READ
                    | windows::Win32::System::Registry::KEY_WRITE,
            ) {
                Ok(reg_key) => {
                    if reg_key.delete_value(name)? {
                        Ok(StepResult::ok(format!("deleted {key}::{name}")))
                    } else {
                        Ok(StepResult::skipped(format!("{key}::{name} was not set")))
                    }
                }
                // The key not existing is the state the deletion wanted.
                Err(_) => Ok(StepResult::skipped(format!("{key} does not exist"))),
            }
        }

        TweakEffect::SetServiceStartup {
            service,
            start_type,
        } => {
            let start = StartType::parse(start_type).ok_or_else(|| Error::Service {
                service: service.clone(),
                source_msg: format!("unrecognised start type `{start_type}`"),
            })?;
            if dry_run {
                return Ok(StepResult::simulated(format!(
                    "would set `{service}` to {}",
                    start.label()
                )));
            }
            if start == StartType::Disabled {
                // Stop it first, or the change only takes effect at reboot.
                let _ = services::stop(service);
            }
            services::set_start_type(service, start)
        }

        TweakEffect::SetOptionalFeature { feature, enabled } => {
            if dry_run {
                return Ok(StepResult::simulated(format!(
                    "would {} the `{feature}` optional feature",
                    if *enabled { "enable" } else { "disable" }
                )));
            }
            run_allowed(
                "dism",
                &[
                    "/online".to_string(),
                    if *enabled {
                        "/enable-feature".to_string()
                    } else {
                        "/disable-feature".to_string()
                    },
                    format!("/featurename:{feature}"),
                    "/norestart".to_string(),
                    "/quiet".to_string(),
                ],
            )
        }

        TweakEffect::SetPowerScheme { scheme } => {
            if dry_run {
                return Ok(StepResult::simulated(format!(
                    "would activate the `{scheme}` power scheme"
                )));
            }
            run_allowed("powercfg", &["/setactive".to_string(), scheme.clone()])
        }

        TweakEffect::RunCommand { program, args } => {
            if let Some(action) = program.strip_prefix(INTERNAL_PREFIX) {
                return run_internal(action, dry_run);
            }
            if dry_run {
                return Ok(StepResult::simulated(format!(
                    "would run `{program} {}`",
                    args.join(" ")
                )));
            }
            run_allowed(program, args)
        }
    }
}

fn run_allowed(program: &str, args: &[String]) -> Result<StepResult> {
    let leaf = program
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(program)
        .trim_end_matches(".exe")
        .to_ascii_lowercase();

    if !ALLOWED_PROGRAMS.contains(&leaf.as_str()) {
        return Err(Error::other(format!(
            "the tweak catalogue tried to run `{program}`, which is not on the allow-list \
             ({}). Tweaks are data and must not be able to execute arbitrary programs.",
            ALLOWED_PROGRAMS.join(", ")
        )));
    }

    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| Error::other(format!("could not run `{program}`: {e}")))?;

    if output.status.success() {
        Ok(StepResult::ok(format!(
            "{program} {} succeeded",
            args.join(" ")
        )))
    } else {
        Err(Error::other(format!(
            "`{program} {}` failed with {:?}: {}",
            args.join(" "),
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// Actions implemented in Rust rather than by shelling out.
fn run_internal(action: &str, dry_run: bool) -> Result<StepResult> {
    match action {
        "clean-temp" => clean_temp(dry_run),
        other => Err(Error::other(format!(
            "unknown internal tweak action `{INTERNAL_PREFIX}{other}`"
        ))),
    }
}

/// Empty the user and system TEMP folders.
///
/// Deletes the *contents*, never the folders themselves — the deletion guard
/// rejects a TEMP directory as a target precisely because other software
/// expects it to exist.
fn clean_temp(dry_run: bool) -> Result<StepResult> {
    let mut freed = 0u64;
    let mut removed = 0usize;
    let mut locked = 0usize;

    let dirs: Vec<std::path::PathBuf> = ["TEMP", "TMP"]
        .iter()
        .filter_map(|var| std::env::var(var).ok())
        .map(std::path::PathBuf::from)
        .chain(
            std::env::var("SystemRoot")
                .ok()
                .map(|r| std::path::PathBuf::from(r).join("Temp")),
        )
        .collect();

    let mut seen: Vec<std::path::PathBuf> = Vec::new();
    for dir in dirs {
        if seen.iter().any(|d| *d == dir) || !dir.is_dir() {
            continue;
        }
        seen.push(dir.clone());

        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if dry_run {
                removed += 1;
                freed += size;
                continue;
            }
            let result = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };
            match result {
                Ok(()) => {
                    removed += 1;
                    freed += size;
                }
                // Files in use are expected and are not a failure.
                Err(_) => locked += 1,
            }
        }
    }

    let detail = format!(
        "{removed} item(s) removed, {locked} in use and skipped, {:.1} MB freed",
        freed as f64 / 1_048_576.0
    );
    Ok(if dry_run {
        StepResult::simulated(detail).with_bytes(freed)
    } else {
        StepResult::ok(detail).with_bytes(freed)
    })
}

/// Apply or revert a whole tweak.
pub fn apply(tweak: &Tweak, enable: bool, dry_run: bool) -> Result<Vec<StepResult>> {
    let effects = if enable { &tweak.apply } else { &tweak.revert };

    if effects.is_empty() {
        return Err(Error::other(format!(
            "`{}` cannot be reverted: it has no revert path",
            tweak.id
        )));
    }

    let mut results = Vec::with_capacity(effects.len());
    for effect in effects {
        results.push(apply_effect(effect, dry_run)?);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dword_values_parse() {
        assert_eq!(
            to_reg_value(RegValueKind::Dword, "1").unwrap(),
            RegValue::Dword(1)
        );
        assert!(to_reg_value(RegValueKind::Dword, "not a number").is_err());
    }

    #[test]
    fn string_values_pass_through() {
        assert_eq!(
            to_reg_value(RegValueKind::String, "506").unwrap(),
            RegValue::Str("506".into())
        );
    }

    #[test]
    fn binary_values_parse_hex_lists() {
        assert_eq!(
            to_reg_value(RegValueKind::Binary, "00,ff,1a").unwrap(),
            RegValue::Binary(vec![0x00, 0xff, 0x1a])
        );
        assert!(to_reg_value(RegValueKind::Binary, "zz").is_err());
    }

    #[test]
    fn arbitrary_programs_are_refused() {
        // A tweak catalogue is data. If data could run anything, a poisoned
        // catalogue update would be remote code execution.
        for program in ["cmd.exe", "powershell", "curl", r"C:\evil\payload.exe"] {
            let err = run_allowed(program, &[]).unwrap_err();
            assert!(
                err.to_string().contains("allow-list"),
                "`{program}` should have been refused, got: {err}"
            );
        }
    }

    #[test]
    fn allowed_programs_pass_the_name_check() {
        // The check itself must accept the real entries; whether the program
        // exists on this host is a separate matter.
        for program in ["powercfg", "powercfg.exe", r"C:\Windows\System32\dism.exe"] {
            let leaf = program
                .rsplit(['\\', '/'])
                .next()
                .unwrap()
                .trim_end_matches(".exe")
                .to_ascii_lowercase();
            assert!(ALLOWED_PROGRAMS.contains(&leaf.as_str()), "{program}");
        }
    }

    #[test]
    fn unknown_internal_actions_are_refused() {
        assert!(run_internal("format-c-drive", true).is_err());
    }

    #[test]
    fn a_dry_run_of_temp_cleaning_deletes_nothing() {
        let result = clean_temp(true).unwrap();
        assert_eq!(result.status, cwico_core::StepStatus::Simulated);
    }

    #[test]
    fn a_tweak_without_a_revert_path_reports_why() {
        let tweak: Tweak = serde_json::from_str(
            r#"{"id":"one.way","category":"cleanup",
                "title":{"en":"t","vi":"t"},"description":{"en":"d","vi":"d"},
                "safety":"caution",
                "apply":[{"op":"runCommand","program":"cwico:clean-temp","args":[]}]}"#,
        )
        .unwrap();
        let err = apply(&tweak, false, true).unwrap_err();
        assert!(err.to_string().contains("no revert path"));
    }
}
