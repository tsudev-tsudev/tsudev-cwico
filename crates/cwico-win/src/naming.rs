//! Name and identifier transforms.
//!
//! Small, pure functions that convert between the several naming schemes
//! Windows uses. They have no Windows dependency on purpose: these are the
//! functions most likely to contain an off-by-one or a wrong assumption, and
//! keeping them host-independent means their tests run in CI on any machine.

/// Derive a package family name from a package full name.
///
/// `DeprovisionPackageForAllUsersAsync` takes a *family* name while
/// `RemovePackageAsync` takes a *full* name, and a plan may carry either.
/// Package names cannot contain `_`, so the first and last underscore-
/// separated components are the name and the publisher id.
///
/// The transform is idempotent: a family name passes through unchanged.
pub fn family_from_full_name(full_or_family: &str) -> String {
    let parts: Vec<&str> = full_or_family.split('_').collect();
    // Full name: Name_Version_Arch[_~]_PublisherId  (4 or 5 parts)
    // Family name: Name_PublisherId                 (2 parts)
    if parts.len() >= 4 {
        format!("{}_{}", parts[0], parts[parts.len() - 1])
    } else {
        full_or_family.to_string()
    }
}

/// Map a `Windows.System.ProcessorArchitecture` value to a label.
pub fn architecture_label(arch: i32) -> &'static str {
    match arch {
        0 => "x86",
        5 => "arm",
        9 => "x64",
        11 => "neutral",
        12 => "arm64",
        _ => "unknown",
    }
}

/// Split a scheduled-task path into its folder and leaf components.
pub fn split_task_path(path: &str) -> (String, String) {
    match path.rfind('\\') {
        Some(0) => ("\\".to_string(), path[1..].to_string()),
        Some(idx) => (path[..idx].to_string(), path[idx + 1..].to_string()),
        None => ("\\".to_string(), path.to_string()),
    }
}

/// Turn a registry path into a filename that is safe on NTFS.
pub fn safe_file_stem(key: &str) -> String {
    let mut out: String = key
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c => c,
        })
        .collect();
    // NTFS caps a path component at 255 characters; leave room for `.reg`
    // and the registry-view suffix.
    if out.len() > 200 {
        out.truncate(200);
    }
    out
}

/// Normalise an `InstallDate`, which is written as `YYYYMMDD` by convention
/// but not by rule. Anything else is dropped rather than guessed at.
pub fn normalise_install_date(raw: &str) -> Option<String> {
    let digits: String = raw.chars().filter(char::is_ascii_digit).collect();
    if digits.len() == 8 {
        Some(format!(
            "{}-{}-{}",
            &digits[0..4],
            &digits[4..6],
            &digits[6..8]
        ))
    } else {
        None
    }
}

/// Windows 11 still reports `Windows 10 Pro` in `ProductName`; build 22000
/// and above is the only reliable discriminator.
pub fn correct_product_name(product: &str, build: Option<u32>) -> String {
    match build {
        Some(n) if n >= 22_000 && product.contains("Windows 10") => {
            product.replace("Windows 10", "Windows 11")
        }
        _ => product.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_name_reduces_to_its_family_name() {
        assert_eq!(
            family_from_full_name("Microsoft.YourPhone_1.24022.83.0_x64__8wekyb3d8bbwe"),
            "Microsoft.YourPhone_8wekyb3d8bbwe"
        );
        assert_eq!(
            family_from_full_name("Microsoft.BingNews_4.55.62231.0_neutral_~_8wekyb3d8bbwe"),
            "Microsoft.BingNews_8wekyb3d8bbwe"
        );
        assert_eq!(
            family_from_full_name("king.com.CandyCrushSaga_1.2420.1.0_x86__kgqvnymyfvs32"),
            "king.com.CandyCrushSaga_kgqvnymyfvs32"
        );
    }

    #[test]
    fn a_family_name_passes_through_unchanged() {
        // Deprovisioning must work whether the plan carried a family name or
        // a full name, so the conversion has to be idempotent.
        let family = "Microsoft.YourPhone_8wekyb3d8bbwe";
        assert_eq!(family_from_full_name(family), family);
        assert_eq!(
            family_from_full_name(&family_from_full_name(
                "Microsoft.YourPhone_1.24022.83.0_x64__8wekyb3d8bbwe"
            )),
            family
        );
    }

    #[test]
    fn architecture_codes_map_to_labels() {
        assert_eq!(architecture_label(9), "x64");
        assert_eq!(architecture_label(0), "x86");
        assert_eq!(architecture_label(12), "arm64");
        assert_eq!(architecture_label(11), "neutral");
        assert_eq!(architecture_label(42), "unknown");
    }

    #[test]
    fn a_root_level_task_splits_to_the_root_folder() {
        let (folder, leaf) = split_task_path(r"\GoogleUpdateTaskMachineUA");
        assert_eq!(folder, "\\");
        assert_eq!(leaf, "GoogleUpdateTaskMachineUA");
    }

    #[test]
    fn a_nested_task_splits_at_the_last_separator() {
        let (folder, leaf) = split_task_path(
            r"\Microsoft\Windows\Customer Experience Improvement Program\Consolidator",
        );
        assert_eq!(
            folder,
            r"\Microsoft\Windows\Customer Experience Improvement Program"
        );
        assert_eq!(leaf, "Consolidator");
    }

    #[test]
    fn a_bare_task_name_is_treated_as_root_level() {
        let (folder, leaf) = split_task_path("SomeTask");
        assert_eq!(folder, "\\");
        assert_eq!(leaf, "SomeTask");
    }

    #[test]
    fn registry_paths_become_safe_filenames() {
        let stem = safe_file_stem(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\App");
        assert!(!stem.contains('\\'));
        assert!(!stem.contains(':'));
        assert!(stem.contains("HKLM") && stem.contains("App"));
    }

    #[test]
    fn guid_keys_survive_sanitisation() {
        let stem = safe_file_stem(r"HKLM\SOFTWARE\...\{90160000-008C-0000-1000-0000000FF1CE}");
        assert!(stem.contains("90160000"));
        // Braces are legal in NTFS filenames and worth keeping: they are how
        // a user recognises which product a backup file belongs to.
        assert!(stem.contains('{'));
    }

    #[test]
    fn very_long_keys_are_truncated_below_the_ntfs_limit() {
        let long = format!(r"HKLM\SOFTWARE\{}", "x".repeat(500));
        assert!(safe_file_stem(&long).len() <= 200);
    }

    #[test]
    fn install_dates_normalise_to_iso() {
        assert_eq!(
            normalise_install_date("20260314").as_deref(),
            Some("2026-03-14")
        );
        assert_eq!(
            normalise_install_date("2026/03/14").as_deref(),
            Some("2026-03-14")
        );
    }

    #[test]
    fn malformed_install_dates_are_dropped_not_guessed() {
        assert_eq!(normalise_install_date("soon"), None);
        assert_eq!(normalise_install_date("2026"), None);
        assert_eq!(normalise_install_date(""), None);
    }

    #[test]
    fn windows_11_is_reported_as_windows_11() {
        assert_eq!(
            correct_product_name("Windows 10 Pro", Some(22_631)),
            "Windows 11 Pro"
        );
        assert_eq!(
            correct_product_name("Windows 10 Pro", Some(19_045)),
            "Windows 10 Pro"
        );
        assert_eq!(
            correct_product_name("Windows 11 Pro", Some(22_631)),
            "Windows 11 Pro"
        );
        assert_eq!(
            correct_product_name("Windows 10 Pro", None),
            "Windows 10 Pro"
        );
    }
}
