//! The process termination allow-list — or rather, deny-list.
//!
//! Step one of the uninstall flow terminates the processes belonging to the
//! software being removed. Two classes of process must never be included,
//! no matter what a safety rule says or what a user selects:
//!
//! * **Shared hosts.** `svchost.exe` runs a dozen unrelated services;
//!   terminating it stops all of them.
//! * **Boot and session critical.** `lsass.exe`, `csrss.exe` and friends take
//!   the machine down or log the user out.
//!
//! Kept host-independent so the list and its check are covered by CI.

/// Processes that are never terminated.
pub const PROTECTED_PROCESSES: &[&str] = &[
    // Shared hosts.
    "svchost.exe",
    "dllhost.exe",
    "rundll32.exe",
    "taskhostw.exe",
    "runtimebroker.exe",
    "backgroundtaskhost.exe",
    "applicationframehost.exe",
    // Boot and session critical.
    "system",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "lsaiso.exe",
    "fontdrvhost.exe",
    "dwm.exe",
    "sihost.exe",
    "ctfmon.exe",
    "explorer.exe",
    "userinit.exe",
    // Security: terminating these disables protection mid-run.
    "msmpeng.exe",
    "securityhealthservice.exe",
    "securityhealthsystray.exe",
    "nissrv.exe",
    "mpdefendercoreservice.exe",
    // Ourselves.
    "cwico.exe",
    "tsudev-cwico.exe",
];

/// `true` when this image must never be terminated.
///
/// Accepts a bare image name or a full path; only the file name is compared,
/// case-insensitively.
pub fn is_protected(image_name: &str) -> bool {
    let leaf = image_name
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(image_name)
        .to_ascii_lowercase();
    PROTECTED_PROCESSES.contains(&leaf.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_hosts_are_protected() {
        for name in [
            "svchost.exe",
            "SVCHOST.EXE",
            "dllhost.exe",
            "RuntimeBroker.exe",
        ] {
            assert!(is_protected(name), "`{name}` must be protected");
        }
    }

    #[test]
    fn boot_critical_processes_are_protected() {
        for name in [
            "lsass.exe",
            "csrss.exe",
            "wininit.exe",
            "winlogon.exe",
            "services.exe",
            "smss.exe",
        ] {
            assert!(is_protected(name), "`{name}` must be protected");
        }
    }

    #[test]
    fn defender_is_protected() {
        assert!(is_protected("MsMpEng.exe"));
        assert!(is_protected("SecurityHealthService.exe"));
    }

    #[test]
    fn a_full_path_is_matched_by_its_leaf() {
        assert!(is_protected(r"C:\Windows\System32\svchost.exe"));
        assert!(is_protected("C:/Windows/System32/lsass.exe"));
    }

    #[test]
    fn the_tool_does_not_terminate_itself() {
        assert!(is_protected("cwico.exe"));
        assert!(is_protected("tsudev-cwico.exe"));
    }

    #[test]
    fn ordinary_applications_are_not_protected() {
        for name in ["OneDrive.exe", "Teams.exe", "msedge.exe", "AcmeLedger.exe"] {
            assert!(!is_protected(name), "`{name}` should be removable");
        }
    }

    #[test]
    fn the_list_is_lowercase_so_the_comparison_works() {
        // `is_protected` lowercases its input and compares against these
        // entries verbatim; an uppercase entry here would silently never match.
        for entry in PROTECTED_PROCESSES {
            assert_eq!(
                *entry,
                entry.to_ascii_lowercase(),
                "`{entry}` must be lowercase in the list"
            );
        }
    }
}
