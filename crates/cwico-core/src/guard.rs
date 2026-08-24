//! Guard rails for destructive filesystem and registry operations.
//!
//! The deep-clean engine deletes directories that a safety rule or an
//! `InstallLocation` pointed at. Those inputs come from the registry, which
//! means they come from software vendors, which means they are occasionally
//! wrong - an `InstallLocation` of `C:\` is a real thing that ships in real
//! products. A single unchecked `remove_dir_all` on such a value destroys the
//! machine.
//!
//! Every path and every registry key therefore passes through this module
//! before anything is removed. The checks are pure string logic so they run
//! identically on the Windows target and in tests on any host.

use crate::error::{Error, Result};

/// Directories that must never be deleted, nor have their parents deleted.
/// Compared case-insensitively after normalisation, with the drive letter
/// replaced by `?:` so the rules hold on any system drive.
const PROTECTED_DIRS: &[&str] = &[
    r"?:",
    r"?:\",
    r"?:\windows",
    r"?:\windows\system32",
    r"?:\windows\syswow64",
    r"?:\windows\winsxs",
    r"?:\windows\system",
    r"?:\windows\fonts",
    r"?:\windows\boot",
    r"?:\windows\inf",
    r"?:\windows\servicing",
    r"?:\windows\assembly",
    r"?:\windows\microsoft.net",
    r"?:\windows\temp",
    r"?:\program files",
    r"?:\program files (x86)",
    r"?:\program files\common files",
    r"?:\program files (x86)\common files",
    r"?:\program files\windowsapps",
    r"?:\programdata",
    r"?:\programdata\microsoft",
    r"?:\programdata\microsoft\windows",
    r"?:\programdata\package cache",
    r"?:\users",
    r"?:\users\public",
    r"?:\users\default",
    r"?:\perflogs",
    r"?:\recovery",
    r"?:\$recycle.bin",
    r"?:\system volume information",
    r"?:\boot",
    r"?:\efi",
];

/// Leaf folder names that are never safe to delete wherever they appear:
/// they are shared containers, not one product's residue. Deleting
/// `...\AppData\Local` takes every application's state with it.
const PROTECTED_LEAF_NAMES: &[&str] = &[
    "appdata",
    "local",
    "locallow",
    "roaming",
    "packages",
    "temp",
    "tmp",
    "programs",
    "start menu",
    "startup",
    "public",
    "default",
    "common files",
    "windowsapps",
];

/// Known folders under a user profile. These hold the user's own documents,
/// so nothing below them is application residue and the whole subtree is off
/// limits - an `InstallLocation` pointing at `C:\Users\me\Documents\App`
/// means the user installed into their documents, not that we may delete it.
///
/// Note this is deliberately narrower than the leaf-name list: a folder
/// *named* `OneDrive` deep under `AppData\Local\Microsoft` is the OneDrive
/// client's own residue and is perfectly safe to remove; `C:\Users\me\OneDrive`
/// is the user's synced files and is not.
const USER_DATA_FOLDERS: &[&str] = &[
    "desktop",
    "documents",
    "downloads",
    "pictures",
    "videos",
    "music",
    "favorites",
    "contacts",
    "links",
    "searches",
    "saved games",
    "3d objects",
    "onedrive",
];

/// Registry hives and keys that must never be deleted.
const PROTECTED_KEYS: &[&str] = &[
    r"hkey_local_machine",
    r"hkey_current_user",
    r"hkey_classes_root",
    r"hkey_users",
    r"hkey_current_config",
    r"hklm",
    r"hkcu",
    r"hkcr",
    r"hku",
    r"hkcc",
    r"hklm\software",
    r"hklm\software\microsoft",
    r"hklm\software\microsoft\windows",
    r"hklm\software\microsoft\windows nt",
    r"hklm\software\microsoft\windows nt\currentversion",
    r"hklm\software\microsoft\windows\currentversion",
    r"hklm\software\wow6432node",
    r"hklm\software\wow6432node\microsoft",
    r"hklm\software\classes",
    r"hklm\system",
    r"hklm\system\currentcontrolset",
    r"hklm\system\currentcontrolset\services",
    r"hklm\system\currentcontrolset\control",
    r"hklm\hardware",
    r"hklm\sam",
    r"hklm\security",
    r"hklm\bcd00000000",
    r"hkcu\software",
    r"hkcu\software\microsoft",
    r"hkcu\software\microsoft\windows",
    r"hkcu\software\microsoft\windows\currentversion",
    r"hkcu\software\classes",
    r"hkcu\control panel",
    r"hkcu\environment",
];

/// Minimum number of path segments below the drive root. `C:\Foo` (1) is
/// rejected; `C:\Program Files\Vendor` (2) is accepted.
const MIN_PATH_DEPTH: usize = 2;

/// Minimum number of key segments below the hive. `HKLM\Software` (1) is
/// rejected; `HKLM\Software\Vendor` (2) is accepted.
const MIN_KEY_DEPTH: usize = 2;

/// Normalise a Windows path for comparison: lowercase, forward slashes folded
/// to backslashes, trailing separators and duplicate separators removed, and
/// the drive letter replaced by `?` so rules are drive-agnostic.
fn normalize_path(raw: &str) -> String {
    let lowered = raw.trim().replace('/', "\\").to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    let mut prev_sep = false;
    for ch in lowered.chars() {
        if ch == '\\' {
            if !prev_sep {
                out.push(ch);
            }
            prev_sep = true;
        } else {
            out.push(ch);
            prev_sep = false;
        }
    }
    while out.len() > 3 && out.ends_with('\\') {
        out.pop();
    }
    // `c:\foo` -> `?:\foo`
    let bytes: Vec<char> = out.chars().collect();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == ':' {
        out.replace_range(0..1, "?");
    }
    out
}

fn normalize_key(raw: &str) -> String {
    let lowered = raw.trim().replace('/', "\\").to_lowercase();
    let trimmed = lowered.trim_end_matches('\\').to_string();
    // Accept both `HKEY_LOCAL_MACHINE\...` and `HKLM\...` spellings.
    let (hive, rest) = match trimmed.split_once('\\') {
        Some((h, r)) => (h, Some(r)),
        None => (trimmed.as_str(), None),
    };
    let canonical_hive = match hive {
        "hkey_local_machine" | "hklm" => "hklm",
        "hkey_current_user" | "hkcu" => "hkcu",
        "hkey_classes_root" | "hkcr" => "hkcr",
        "hkey_users" | "hku" => "hku",
        "hkey_current_config" | "hkcc" => "hkcc",
        other => other,
    };
    match rest {
        Some(r) => format!("{canonical_hive}\\{r}"),
        None => canonical_hive.to_string(),
    }
}

fn segments(normalized: &str) -> Vec<&str> {
    normalized
        .split('\\')
        .filter(|s| !s.is_empty() && *s != "?:")
        .collect()
}

/// Reject a filesystem path that must not be deleted.
///
/// The input may still contain unexpanded `%VARS%`; expand them first, because
/// an unexpanded variable collapses to a short, dangerous path.
pub fn validate_delete_path(raw: &str) -> Result<()> {
    let reject = |reason: &str| {
        Err(Error::UnsafeDeleteTarget {
            path: raw.into(),
            reason: reason.to_string(),
        })
    };

    if raw.trim().is_empty() {
        return reject("empty path");
    }
    if raw.contains('%') {
        return reject("contains an unexpanded environment variable");
    }
    if raw.contains("..") {
        return reject("contains a parent-directory traversal");
    }
    if raw.contains('*') || raw.contains('?') {
        return reject("contains a wildcard; deep clean deletes exact paths only");
    }

    let norm = normalize_path(raw);

    // UNC and device paths are out of scope and too easy to get wrong.
    if norm.starts_with("\\\\") {
        return reject("UNC and device paths are not deletable targets");
    }
    if !(norm.len() >= 3 && norm.starts_with("?:\\")) {
        return reject("not an absolute path on a local drive");
    }

    for protected in PROTECTED_DIRS {
        if norm == *protected {
            return reject("protected system directory");
        }
        // Deleting a parent of a protected directory takes it with it.
        if protected.starts_with(&format!("{norm}\\")) {
            return reject("is a parent of a protected system directory");
        }
    }

    let segs = segments(&norm);
    if segs.len() < MIN_PATH_DEPTH {
        return reject("too close to the drive root");
    }

    if let Some(leaf) = segs.last() {
        if PROTECTED_LEAF_NAMES.contains(leaf) {
            return reject("targets a user-data or shared system folder");
        }
    }

    // `C:\Users\<name>` and `C:\Users\<name>\AppData\<Local|Roaming>` are
    // containers, not residue; only things below them may be removed.
    if segs.first() == Some(&"users") {
        if segs.len() < 4 {
            return reject("targets a user profile root or a known folder");
        }
        if segs.len() == 4 && segs[2] == "appdata" {
            return reject("targets an AppData root");
        }
        // Anything under Documents, Desktop, OneDrive… is the user's data.
        if USER_DATA_FOLDERS.contains(&segs[2]) {
            return reject("targets the user's own data folders");
        }
    }

    Ok(())
}

/// Reject a registry key that must not be deleted.
pub fn validate_delete_key(raw: &str) -> Result<()> {
    let reject = |reason: &str| {
        Err(Error::UnsafeDeleteTarget {
            path: raw.into(),
            reason: reason.to_string(),
        })
    };

    if raw.trim().is_empty() {
        return reject("empty registry key");
    }
    if raw.contains('*') || raw.contains('?') {
        return reject("contains a wildcard; deep clean deletes exact keys only");
    }

    let norm = normalize_key(raw);

    for protected in PROTECTED_KEYS {
        if norm == *protected {
            return reject("protected registry key");
        }
        if protected.starts_with(&format!("{norm}\\")) {
            return reject("is a parent of a protected registry key");
        }
    }

    let segs: Vec<&str> = norm.split('\\').filter(|s| !s.is_empty()).collect();
    if segs.len() < MIN_KEY_DEPTH + 1 {
        return reject("too close to a hive root");
    }

    // The service control database is off limits at every depth. A service
    // key *is* the service; a subkey under it holds the parameters the
    // service needs to start. Services are turned off through the service
    // control manager, which is reversible - deep clean has no business here.
    if norm.starts_with(r"hklm\system\currentcontrolset\services") {
        return reject(
            "targets the service control database; disable the service instead of deleting it",
        );
    }

    // Likewise the driver and class registration trees.
    if norm.starts_with(r"hklm\system\currentcontrolset\control\class")
        || norm.starts_with(r"hklm\system\currentcontrolset\enum")
    {
        return reject("targets the device and driver registration database");
    }

    Ok(())
}

/// Validate a `KEY::ValueName` pair used by `DeepCleanRegistry`.
pub fn validate_delete_value(raw: &str) -> Result<(String, String)> {
    let Some((key, value)) = raw.split_once("::") else {
        return Err(Error::UnsafeDeleteTarget {
            path: raw.into(),
            reason: "expected `KEY::ValueName`".into(),
        });
    };
    if value.trim().is_empty() {
        return Err(Error::UnsafeDeleteTarget {
            path: raw.into(),
            reason: "empty value name".into(),
        });
    }
    // Deleting a *value* is far less destructive than deleting a key, so the
    // depth rule is relaxed - but the hive roots are still off limits.
    let norm = normalize_key(key);
    let segs: Vec<&str> = norm.split('\\').filter(|s| !s.is_empty()).collect();
    if segs.len() < 2 {
        return Err(Error::UnsafeDeleteTarget {
            path: raw.into(),
            reason: "value lives directly under a hive root".into(),
        });
    }
    Ok((key.to_string(), value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn assert_rejected(path: &str) {
        assert!(
            validate_delete_path(path).is_err(),
            "`{path}` must be rejected as a delete target"
        );
    }

    #[track_caller]
    fn assert_allowed(path: &str) {
        validate_delete_path(path)
            .unwrap_or_else(|e| panic!("`{path}` should be deletable, got: {e}"));
    }

    #[test]
    fn drive_roots_and_system_dirs_are_rejected() {
        for p in [
            r"C:\",
            r"C:",
            r"D:\",
            r"C:\Windows",
            r"C:\Windows\System32",
            r"C:\Windows\SysWOW64",
            r"C:\Windows\WinSxS",
            r"C:\Program Files",
            r"C:\Program Files (x86)",
            r"C:\ProgramData",
            r"C:\Users",
            r"C:\Users\Public",
            r"C:\$Recycle.Bin",
            r"C:\System Volume Information",
        ] {
            assert_rejected(p);
        }
    }

    #[test]
    fn case_and_separator_variations_do_not_bypass_the_guard() {
        for p in [
            r"c:\windows\system32",
            r"C:/Windows/System32",
            r"C:\WINDOWS\\SYSTEM32\",
            r"C:\Windows\System32\",
            r"c:/WINDOWS/system32//",
        ] {
            assert_rejected(p);
        }
    }

    #[test]
    fn a_folder_named_like_a_known_folder_is_still_residue_when_deep_in_appdata() {
        // The distinction the guard has to get right: the user's synced
        // OneDrive files are untouchable, the OneDrive client's own state
        // folder is exactly what deep clean exists to remove.
        assert_rejected(r"C:\Users\tsudev\OneDrive");
        assert_rejected(r"C:\Users\tsudev\OneDrive\Work");
        assert_allowed(r"C:\Users\tsudev\AppData\Local\Microsoft\OneDrive");
        assert_allowed(r"C:\ProgramData\Microsoft OneDrive");
    }

    #[test]
    fn shared_containers_are_rejected_but_their_children_are_not() {
        assert_rejected(r"C:\Users\tsudev\AppData\Local\Packages");
        assert_allowed(r"C:\Users\tsudev\AppData\Local\Packages\Microsoft.YourPhone_8wekyb3d8bbwe");
        assert_rejected(r"C:\Users\tsudev\AppData\Local\Temp");
        assert_rejected(r"C:\Users\tsudev\AppData\Roaming\Microsoft\Windows\Start Menu\Programs");
    }

    #[test]
    fn anything_under_the_users_own_data_folders_is_rejected() {
        for p in [
            r"C:\Users\tsudev\Documents\MyApp",
            r"C:\Users\tsudev\Desktop\Installer",
            r"C:\Users\tsudev\Downloads\vendor\cache",
            r"C:\Users\tsudev\Pictures\App Cache",
        ] {
            assert_rejected(p);
        }
    }

    #[test]
    fn user_profile_roots_and_known_folders_are_rejected() {
        for p in [
            r"C:\Users\tsudev",
            r"C:\Users\tsudev\Desktop",
            r"C:\Users\tsudev\Documents",
            r"C:\Users\tsudev\Downloads",
            r"C:\Users\tsudev\AppData",
            r"C:\Users\tsudev\AppData\Local",
            r"C:\Users\tsudev\AppData\Roaming",
            r"C:\Users\tsudev\OneDrive",
        ] {
            assert_rejected(p);
        }
    }

    #[test]
    fn genuine_application_residue_is_allowed() {
        for p in [
            r"C:\Users\tsudev\AppData\Local\Microsoft\OneDrive",
            r"C:\Users\tsudev\AppData\Roaming\Skype",
            r"C:\Program Files\Vendor\Product",
            r"C:\Program Files (x86)\Vendor\Product",
            r"C:\ProgramData\Microsoft OneDrive",
            r"D:\Games\SomeGame",
        ] {
            assert_allowed(p);
        }
    }

    #[test]
    fn unexpanded_variables_and_traversal_are_rejected() {
        assert_rejected(r"%LOCALAPPDATA%\Microsoft\OneDrive");
        assert_rejected(r"C:\Program Files\Vendor\..\..\Windows");
        assert_rejected(r"C:\Program Files\*");
        assert_rejected(r"\\server\share\folder");
        assert_rejected("");
        assert_rejected("   ");
    }

    #[test]
    fn a_parent_of_a_protected_dir_is_rejected() {
        // Deleting C:\Windows would take System32 with it; both are listed,
        // but the parent rule is what catches unlisted intermediates.
        assert_rejected(r"C:\Program Files\Common Files");
    }

    #[test]
    fn relative_paths_are_rejected() {
        assert_rejected(r"Windows\System32");
        assert_rejected(r"\Windows");
        assert_rejected("just-a-name");
    }

    #[track_caller]
    fn assert_key_rejected(key: &str) {
        assert!(
            validate_delete_key(key).is_err(),
            "`{key}` must be rejected as a registry delete target"
        );
    }

    #[test]
    fn hive_roots_and_core_keys_are_rejected() {
        for k in [
            r"HKLM",
            r"HKEY_LOCAL_MACHINE",
            r"HKLM\SOFTWARE",
            r"HKLM\SOFTWARE\Microsoft",
            r"HKLM\SOFTWARE\Microsoft\Windows",
            r"HKLM\SYSTEM\CurrentControlSet",
            r"HKLM\SYSTEM\CurrentControlSet\Services",
            r"HKEY_CURRENT_USER\Software",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion",
            r"HKLM\SAM",
            r"HKLM\SECURITY",
        ] {
            assert_key_rejected(k);
        }
    }

    #[test]
    fn the_service_database_is_rejected_at_every_depth() {
        for k in [
            r"HKLM\SYSTEM\CurrentControlSet\Services",
            r"HKLM\SYSTEM\CurrentControlSet\Services\Fax",
            r"HKLM\SYSTEM\CurrentControlSet\Services\WinDefend",
            r"HKLM\SYSTEM\CurrentControlSet\Services\Foo\Parameters",
            r"hklm\system\currentcontrolset\services\bar\deep\nested",
        ] {
            assert_key_rejected(k);
        }
    }

    #[test]
    fn the_device_registration_database_is_rejected() {
        assert_key_rejected(r"HKLM\SYSTEM\CurrentControlSet\Enum\PCI\VEN_10DE");
        assert_key_rejected(
            r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
        );
    }

    #[test]
    fn vendor_keys_are_allowed() {
        for k in [
            r"HKCU\Software\Microsoft\OneDrive",
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\MyApp",
            r"HKCU\Software\Vendor\Product",
            r"HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Vendor\Product",
        ] {
            validate_delete_key(k).unwrap_or_else(|e| panic!("`{k}` should be deletable: {e}"));
        }
    }

    #[test]
    fn hklm_and_hkey_local_machine_spellings_are_equivalent() {
        assert!(validate_delete_key(r"HKEY_LOCAL_MACHINE\SOFTWARE").is_err());
        assert!(validate_delete_key(r"hklm\software").is_err());
    }

    #[test]
    fn registry_values_parse_and_reject_hive_level_writes() {
        let (key, value) =
            validate_delete_value(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run::OneDrive")
                .unwrap();
        assert!(key.ends_with("Run"));
        assert_eq!(value, "OneDrive");

        assert!(validate_delete_value(r"HKCU\Software\Microsoft\Run").is_err());
        assert!(validate_delete_value(r"HKLM::Something").is_err());
        assert!(validate_delete_value(r"HKCU\Software\Vendor::").is_err());
    }
}
