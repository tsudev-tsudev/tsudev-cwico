//! Autostart entries: the `Run` keys and the Startup folders.
//!
//! What a user experiences as "this thing keeps launching itself" is almost
//! always one of these six locations. They are also the residue an uninstaller
//! most often forgets, leaving a startup entry that points at a program that
//! no longer exists.

use crate::registry::{RegKey, RegValue, RegView};
use cwico_core::backend::StepResult;
use cwico_core::Result;
use std::path::PathBuf;
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

/// The registry autostart locations, as `(hive label, subkey, view)`.
pub const RUN_KEYS: &[(&str, &str, RegView)] = &[
    (
        "HKLM",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
        RegView::Bits64,
    ),
    (
        "HKLM",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
        RegView::Bits32,
    ),
    (
        "HKLM",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce",
        RegView::Bits64,
    ),
    (
        "HKCU",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
        RegView::Bits64,
    ),
    (
        "HKCU",
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce",
        RegView::Bits64,
    ),
];

/// One autostart entry.
#[derive(Debug, Clone)]
pub struct StartupEntry {
    /// The value name, or the shortcut's file name.
    pub name: String,
    /// The command or target it launches.
    pub command: String,
    /// `HKCU\Software\...\Run`, or the Startup folder path.
    pub location: String,
    /// `true` when the entry points at a file that no longer exists - pure
    /// residue, and always safe to remove.
    pub target_missing: bool,
    /// `true` for a Startup-folder shortcut rather than a registry value.
    pub is_shortcut: bool,
}

/// Extract the executable path from a command line so we can check it exists.
fn target_path(command: &str) -> Option<PathBuf> {
    let tokens = crate::cmdline::tokenize(command);
    tokens.first().map(PathBuf::from)
}

/// Enumerate registry autostart values and Startup-folder shortcuts.
pub fn enumerate() -> Result<Vec<StartupEntry>> {
    let mut out = Vec::new();

    for (hive_label, subkey, view) in RUN_KEYS {
        let root = if *hive_label == "HKLM" {
            HKEY_LOCAL_MACHINE
        } else {
            HKEY_CURRENT_USER
        };

        let Ok(key) = RegKey::open(root, subkey, *view, KEY_READ) else {
            continue;
        };

        for (name, value) in key.values() {
            let command = match &value {
                RegValue::Str(s) | RegValue::ExpandStr(s) => s.clone(),
                other => other.as_string().unwrap_or_default(),
            };
            if command.trim().is_empty() {
                continue;
            }
            let target_missing = target_path(&command).map(|p| !p.exists()).unwrap_or(false);

            out.push(StartupEntry {
                name,
                command,
                location: format!("{hive_label}\\{subkey}"),
                target_missing,
                is_shortcut: false,
            });
        }
    }

    for folder in startup_folders() {
        let Ok(entries) = std::fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .is_none_or(|e| !e.eq_ignore_ascii_case("lnk") && !e.eq_ignore_ascii_case("url"))
            {
                continue;
            }
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            out.push(StartupEntry {
                name,
                command: path.to_string_lossy().into_owned(),
                location: folder.to_string_lossy().into_owned(),
                // Resolving a .lnk needs IShellLink; the shortcut file itself
                // existing is what matters for removal.
                target_missing: false,
                is_shortcut: true,
            });
        }
    }

    out.sort_by_key(|e| e.name.to_lowercase());
    Ok(out)
}

/// The per-user and all-users Startup folders.
pub fn startup_folders() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(appdata) = std::env::var("APPDATA") {
        out.push(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup"));
    }
    if let Ok(programdata) = std::env::var("ProgramData") {
        out.push(PathBuf::from(programdata).join(r"Microsoft\Windows\Start Menu\Programs\StartUp"));
    }
    out
}

/// Remove one autostart entry.
///
/// `location` is either a registry key path or a Startup folder path; the
/// distinction is what decides between deleting a value and deleting a file.
pub fn remove(location: &str, name: &str) -> Result<StepResult> {
    if location.starts_with("HK") {
        let mut removed = false;
        for view in [RegView::Bits64, RegView::Bits32] {
            if let Ok(key) = RegKey::open_path(
                location,
                view,
                windows::Win32::System::Registry::KEY_READ
                    | windows::Win32::System::Registry::KEY_WRITE,
            ) {
                if key.delete_value(name)? {
                    removed = true;
                }
            }
        }
        return Ok(if removed {
            StepResult::ok(format!("removed autostart value `{name}` from {location}"))
        } else {
            StepResult::skipped(format!("autostart value `{name}` was not present"))
        });
    }

    // Startup folder: remove the shortcut, whichever extension it uses.
    let folder = PathBuf::from(location);
    for extension in ["lnk", "url"] {
        let candidate = folder.join(format!("{name}.{extension}"));
        if candidate.exists() {
            std::fs::remove_file(&candidate).map_err(|e| cwico_core::Error::io(&candidate, e))?;
            return Ok(StepResult::ok(format!(
                "removed startup shortcut `{}`",
                candidate.display()
            )));
        }
    }

    Ok(StepResult::skipped(format!(
        "startup shortcut `{name}` was not present in {location}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_registry_views_of_the_run_key_are_covered() {
        let hklm_run: Vec<RegView> = RUN_KEYS
            .iter()
            .filter(|(hive, key, _)| *hive == "HKLM" && key.ends_with(r"\Run"))
            .map(|(_, _, view)| *view)
            .collect();
        assert!(hklm_run.contains(&RegView::Bits64));
        assert!(hklm_run.contains(&RegView::Bits32));
    }

    #[test]
    fn both_hives_are_covered() {
        assert!(RUN_KEYS.iter().any(|(hive, _, _)| *hive == "HKLM"));
        assert!(RUN_KEYS.iter().any(|(hive, _, _)| *hive == "HKCU"));
    }

    #[test]
    fn runonce_keys_are_included() {
        assert!(RUN_KEYS.iter().any(|(_, key, _)| key.ends_with("RunOnce")));
    }

    #[test]
    fn a_quoted_command_yields_its_executable() {
        let path = target_path(r#""C:\Program Files\App\app.exe" --background"#).unwrap();
        assert_eq!(path, PathBuf::from(r"C:\Program Files\App\app.exe"));
    }

    #[test]
    fn an_unquoted_command_yields_its_executable() {
        let path = target_path(r"C:\App\app.exe -silent").unwrap();
        assert_eq!(path, PathBuf::from(r"C:\App\app.exe"));
    }
}
