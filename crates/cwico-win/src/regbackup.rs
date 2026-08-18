//! Registry backup: exporting `.reg` files before anything is changed.
//!
//! The second line of rollback, and the more useful one in practice. A System
//! Restore Point is all-or-nothing and can be days old by the time the user
//! notices a problem; a `.reg` file is a surgical undo the user can
//! double-click.
//!
//! `reg.exe export` is used rather than `RegSaveKeyEx` on purpose:
//! `RegSaveKeyEx` writes a binary hive that only `RegRestoreKey` understands,
//! while a `.reg` file is plain text a user can read, diff and re-import from
//! Explorer without this tool being installed at all.

use crate::naming::safe_file_stem;
use crate::registry::RegView;
use cwico_core::backend::RegistryBackup;
use cwico_core::{Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Export one key to `<out_dir>/<sanitised key>.reg`.
///
/// A key that does not exist is **not** an error: the planner collects keys
/// optimistically, and a missing key simply has nothing to back up.
pub fn export_key(key: &str, out_dir: &Path, view: RegView) -> Result<Option<RegistryBackup>> {
    std::fs::create_dir_all(out_dir).map_err(|e| Error::io(out_dir, e))?;

    let file: PathBuf = out_dir.join(format!("{}-{}.reg", safe_file_stem(key), view.label()));

    let output = Command::new("reg.exe")
        .arg("export")
        .arg(key)
        .arg(&file)
        .arg("/y")
        .arg(match view {
            RegView::Bits64 => "/reg:64",
            RegView::Bits32 => "/reg:32",
        })
        .output()
        .map_err(|e| Error::RegistryBackup {
            key: key.to_string(),
            source_msg: format!("could not run reg.exe: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // reg.exe says "The system was unable to find the specified registry
        // key or value" for a key that is simply absent.
        let missing = stderr.contains("unable to find")
            || stderr.contains("không tìm thấy")
            || output.status.code() == Some(1);
        if missing {
            tracing::debug!(key, "nothing to back up: the key does not exist");
            return Ok(None);
        }
        return Err(Error::RegistryBackup {
            key: key.to_string(),
            source_msg: format!(
                "reg.exe export failed ({:?}): {}",
                output.status.code(),
                stderr.trim()
            ),
        });
    }

    let bytes = std::fs::metadata(&file).map(|m| m.len()).unwrap_or(0);
    if bytes == 0 {
        // An empty export is not a backup; do not let it look like one.
        let _ = std::fs::remove_file(&file);
        return Ok(None);
    }

    tracing::info!(key, file = %file.display(), bytes, "registry key exported");
    Ok(Some(RegistryBackup {
        key: key.to_string(),
        file,
        bytes,
    }))
}

/// Export every key, in both registry views, skipping the ones that do not
/// exist. Returns what was actually written.
///
/// One key failing does not abort the batch: the engine records the failure
/// as a run warning and the user decides whether to continue.
pub fn export_all(keys: &[String], out_dir: &Path) -> Result<Vec<RegistryBackup>> {
    let mut out = Vec::new();
    let mut failures = Vec::new();

    for key in keys {
        for view in [RegView::Bits64, RegView::Bits32] {
            match export_key(key, out_dir, view) {
                Ok(Some(backup)) => out.push(backup),
                Ok(None) => {}
                Err(e) => failures.push(format!("{key}: {e}")),
            }
        }
    }

    if out.is_empty() && !failures.is_empty() {
        return Err(Error::RegistryBackup {
            key: format!("{} key(s)", keys.len()),
            source_msg: failures.join("; "),
        });
    }
    for failure in &failures {
        tracing::warn!(detail = %failure, "a registry key could not be exported");
    }

    Ok(out)
}

/// Write a small `restore.cmd` next to the exports so the user can undo the
/// whole run without this tool.
pub fn write_restore_script(out_dir: &Path, backups: &[RegistryBackup]) -> Result<PathBuf> {
    let path = out_dir.join("restore-registry.cmd");
    let mut script = String::from(
        "@echo off\r\n\
         REM ---------------------------------------------------------------\r\n\
         REM  tsudev-cwico -- registry rollback\r\n\
         REM  https://tsudev.com\r\n\
         REM\r\n\
         REM  Re-imports every registry key this run backed up.\r\n\
         REM  Right-click this file and choose \"Run as administrator\".\r\n\
         REM ---------------------------------------------------------------\r\n\
         setlocal\r\n\
         echo Restoring registry keys backed up by tsudev-cwico...\r\n\r\n",
    );

    for backup in backups {
        let name = backup
            .file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        script.push_str(&format!("echo   {}\r\n", backup.key));
        script.push_str(&format!("reg.exe import \"%~dp0{name}\"\r\n"));
    }

    script.push_str("\r\necho.\r\necho Done. A restart is recommended.\r\npause\r\n");

    std::fs::write(&path, script).map_err(|e| Error::io(&path, e))?;
    Ok(path)
}
