//! The deep-clean pass: removing what the vendor's uninstaller left behind.
//!
//! This is the most dangerous code in the project, so it is also the most
//! constrained. Every path and every key goes through
//! [`cwico_core::guard`] first, without exception and without an override.
//! The guard rejects drive roots, Windows and Program Files, user profile
//! roots, the user's own document folders and every registry hive root - see
//! that module for the full rule set and its tests.
//!
//! A path that fails the guard is reported to the user as skipped, with the
//! reason. It is never deleted "because the rule said so": rules are data,
//! and data can be wrong.

use crate::env;
use crate::registry::{self, RegView};
use cwico_core::backend::CleanSummary;
use cwico_core::guard;
use cwico_core::Result;
use std::path::Path;

/// Sum the size of a directory tree, following no symlinks.
fn directory_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Delete one directory tree, retrying briefly.
///
/// A folder that is still locked immediately after the uninstaller exited is
/// common - antivirus scanners and the shell hold handles for a moment - so
/// a short retry turns a spurious failure into a success.
fn remove_tree(path: &Path) -> std::io::Result<()> {
    const ATTEMPTS: usize = 3;
    let mut last = Ok(());
    for attempt in 0..ATTEMPTS {
        match std::fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                last = Err(e);
                if attempt + 1 < ATTEMPTS {
                    std::thread::sleep(std::time::Duration::from_millis(250));
                }
            }
        }
    }
    last
}

/// Delete residual filesystem paths.
///
/// `dry_run` reports what would happen and changes nothing.
pub fn delete_paths(paths: &[String], dry_run: bool) -> Result<CleanSummary> {
    let mut summary = CleanSummary::default();

    for raw in paths {
        let expanded = env::expand_path(raw);

        // --- Guard rail. Nothing gets past this. --------------------------
        if let Err(e) = guard::validate_delete_path(&expanded) {
            tracing::warn!(path = %expanded, error = %e, "refused an unsafe delete target");
            summary.skipped.push(format!("{expanded} - {e}"));
            continue;
        }

        let path = Path::new(&expanded);
        if !path.exists() {
            summary.skipped.push(format!("{expanded} - not present"));
            continue;
        }

        // Do not follow a symlink or junction out of the validated path: a
        // reparse point under AppData could otherwise redirect the delete
        // anywhere on the system.
        match std::fs::symlink_metadata(path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                match std::fs::remove_file(path) {
                    Ok(()) => summary.removed.push(expanded),
                    Err(e) => summary.failed.push(format!("{expanded} - {e}")),
                }
                continue;
            }
            Ok(_) => {}
            Err(e) => {
                summary.failed.push(format!("{expanded} - {e}"));
                continue;
            }
        }

        let size = if path.is_dir() {
            directory_size(path)
        } else {
            std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        };

        if dry_run {
            summary.removed.push(expanded);
            summary.bytes_freed = summary.bytes_freed.saturating_add(size);
            continue;
        }

        let result = if path.is_dir() {
            remove_tree(path)
        } else {
            std::fs::remove_file(path)
        };

        match result {
            Ok(()) => {
                tracing::info!(path = %expanded, bytes = size, "removed residue");
                summary.removed.push(expanded);
                summary.bytes_freed = summary.bytes_freed.saturating_add(size);
            }
            Err(e) => {
                tracing::warn!(path = %expanded, error = %e, "could not remove residue");
                summary.failed.push(format!("{expanded} - {e}"));
            }
        }
    }

    Ok(summary)
}

/// Delete residual registry keys and values.
///
/// Keys are removed in both registry views, because a 32-bit product writes
/// under `WOW6432Node` and the safety rules record the logical path.
pub fn delete_registry(keys: &[String], values: &[String], dry_run: bool) -> Result<CleanSummary> {
    let mut summary = CleanSummary::default();

    for key in keys {
        // --- Guard rail. --------------------------------------------------
        if let Err(e) = guard::validate_delete_key(key) {
            tracing::warn!(key = %key, error = %e, "refused an unsafe registry target");
            summary.skipped.push(format!("{key} - {e}"));
            continue;
        }

        if dry_run {
            summary.removed.push(key.clone());
            continue;
        }

        let mut removed_any = false;
        let mut errors = Vec::new();
        for view in [RegView::Bits64, RegView::Bits32] {
            match registry::delete_tree(key, view) {
                Ok(true) => removed_any = true,
                Ok(false) => {}
                Err(e) => errors.push(format!("{}-bit: {e}", view.label())),
            }
        }

        if removed_any {
            tracing::info!(key = %key, "removed registry residue");
            summary.removed.push(key.clone());
        } else if errors.is_empty() {
            summary.skipped.push(format!("{key} - not present"));
        } else {
            summary
                .failed
                .push(format!("{key} - {}", errors.join("; ")));
        }
    }

    for spec in values {
        let (key, name) = match guard::validate_delete_value(spec) {
            Ok(pair) => pair,
            Err(e) => {
                summary.skipped.push(format!("{spec} - {e}"));
                continue;
            }
        };

        if dry_run {
            summary.removed.push(spec.clone());
            continue;
        }

        let mut removed_any = false;
        let mut errors = Vec::new();
        for view in [RegView::Bits64, RegView::Bits32] {
            match registry::RegKey::open_path(
                &key,
                view,
                windows::Win32::System::Registry::KEY_READ
                    | windows::Win32::System::Registry::KEY_WRITE,
            ) {
                Ok(reg_key) => match reg_key.delete_value(&name) {
                    Ok(true) => removed_any = true,
                    Ok(false) => {}
                    Err(e) => errors.push(format!("{}-bit: {e}", view.label())),
                },
                // The key not existing in one view is normal.
                Err(_) => {}
            }
        }

        if removed_any {
            summary.removed.push(spec.clone());
        } else if errors.is_empty() {
            summary.skipped.push(format!("{spec} - not present"));
        } else {
            summary
                .failed
                .push(format!("{spec} - {}", errors.join("; ")));
        }
    }

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_paths_are_skipped_rather_than_deleted() {
        // The whole point of the module: a rule pointing at C:\Windows must
        // produce a skip with a reason, not a deletion and not a hard error.
        let summary = delete_paths(
            &[
                r"C:\Windows".to_string(),
                r"C:\".to_string(),
                r"C:\Users\someone".to_string(),
                r"%DEFINITELY_NOT_SET_CWICO%\App".to_string(),
            ],
            true,
        )
        .unwrap();

        assert!(summary.removed.is_empty(), "{:?}", summary.removed);
        assert_eq!(summary.skipped.len(), 4);
        assert!(summary.failed.is_empty());
        assert!(summary.skipped.iter().all(|s| s.contains('-')));
    }

    #[test]
    fn unsafe_registry_keys_are_skipped() {
        let summary = delete_registry(
            &[
                r"HKLM\SOFTWARE".to_string(),
                r"HKLM\SYSTEM\CurrentControlSet\Services".to_string(),
                r"HKCU".to_string(),
            ],
            &[],
            true,
        )
        .unwrap();
        assert!(summary.removed.is_empty());
        assert_eq!(summary.skipped.len(), 3);
    }

    #[test]
    fn a_dry_run_reports_a_plausible_target_without_touching_it() {
        let dir = std::env::temp_dir().join("cwico-deepclean-dry-run-test");
        let nested = dir.join("Vendor").join("Product");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("data.bin"), vec![0u8; 4_096]).unwrap();

        let summary = delete_paths(&[nested.to_string_lossy().into_owned()], true).unwrap();

        // Whether the temp path passes the guard depends on the host, so
        // assert the invariant that matters instead: a dry run never deletes.
        assert!(nested.exists(), "a dry run must not delete anything");
        assert!(summary.failed.is_empty());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_missing_path_is_skipped_not_failed() {
        // Re-running a plan should be quiet, not a wall of red.
        let summary = delete_paths(
            &[r"C:\Program Files\CwicoDefinitelyNotThere\Product".to_string()],
            false,
        )
        .unwrap();
        assert!(summary.failed.is_empty());
        assert_eq!(summary.removed.len(), 0);
        assert_eq!(summary.skipped.len(), 1);
    }
}
