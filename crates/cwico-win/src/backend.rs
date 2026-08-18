//! [`WindowsBackend`]: the real implementation of [`PlatformBackend`].
//!
//! Every method here is a thin adapter between the engine's vocabulary and
//! one of this crate's modules. The interesting logic lives in those modules;
//! what this file is responsible for is the contract the engine relies on:
//!
//! * **Idempotence.** "Already gone" is [`StepStatus::Skipped`], not an error,
//!   so re-running the same plan produces a quiet second run.
//! * **No hidden work.** A method does what its name says and nothing else;
//!   the ordering and the safety gate belong to the engine and the planner.

use crate::{
    appx, deepclean, elevation, env, exec, process, regbackup, restore, scanner, services, startup,
    tasks,
};
use cwico_core::backend::{
    CleanSummary, EventSink, ExecOutcome, KilledProcess, PlatformBackend, PlatformInfo,
    RegistryBackup, RestorePointInfo, StepResult,
};
use cwico_core::safety::SafetyDatabase;
use cwico_core::scan::{ScanOptions, ScanReport};
use cwico_core::Result;
use std::path::Path;
use std::time::Duration;

/// How long to wait for terminated processes to actually exit before running
/// the uninstaller. Short: the kill already succeeded, this only covers the
/// kernel closing handles.
const PROCESS_EXIT_GRACE: Duration = Duration::from_secs(5);

/// The Windows implementation of the platform backend.
#[derive(Debug, Default)]
pub struct WindowsBackend {
    /// Cached at construction: the elevation state cannot change within the
    /// lifetime of a process, and the engine asks for it repeatedly.
    elevated: bool,
}

impl WindowsBackend {
    pub fn new() -> Self {
        Self {
            elevated: elevation::is_elevated(),
        }
    }
}

/// Read the OS description from the registry rather than `GetVersionEx`,
/// which lies to unmanifested processes for compatibility reasons.
fn os_description() -> (String, Option<String>) {
    use crate::registry::{RegKey, RegView};
    use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, KEY_READ};

    let Ok(key) = RegKey::open(
        HKEY_LOCAL_MACHINE,
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
        RegView::Bits64,
        KEY_READ,
    ) else {
        return ("Windows".into(), None);
    };

    let product = key
        .string("ProductName")
        .unwrap_or_else(|| "Windows".into());
    let display_version = key.string("DisplayVersion");
    let build = key.string("CurrentBuild");
    let ubr = key.u32("UBR");

    let full_build = match (&build, ubr) {
        (Some(b), Some(u)) => Some(format!("{b}.{u}")),
        (Some(b), None) => Some(b.clone()),
        _ => None,
    };

    let product = crate::naming::correct_product_name(
        &product,
        build.as_deref().and_then(|b| b.parse::<u32>().ok()),
    );

    let description = match (&display_version, &full_build) {
        (Some(v), Some(b)) => format!("{product} {v} (build {b})"),
        (None, Some(b)) => format!("{product} (build {b})"),
        _ => product,
    };

    (description, full_build)
}

impl PlatformBackend for WindowsBackend {
    fn platform_info(&self) -> PlatformInfo {
        let (os_description, os_build) = os_description();
        PlatformInfo {
            platform: "windows".into(),
            os_description,
            os_build,
            arch: std::env::consts::ARCH.to_string(),
            elevated: self.elevated,
            system_restore_available: restore::is_available(),
        }
    }

    fn is_elevated(&self) -> bool {
        self.elevated
    }

    fn scan(
        &self,
        options: &ScanOptions,
        db: &SafetyDatabase,
        sink: &dyn EventSink,
    ) -> Result<ScanReport> {
        scanner::scan(options, db, sink, self.elevated)
    }

    // -- Safety scaffolding --------------------------------------------------

    fn create_restore_point(&self, description: &str) -> Result<RestorePointInfo> {
        restore::create(description)
    }

    fn backup_registry_keys(&self, keys: &[String], out_dir: &Path) -> Result<Vec<RegistryBackup>> {
        let backups = regbackup::export_all(keys, out_dir)?;
        // A rollback the user can perform without this tool installed.
        if !backups.is_empty() {
            match regbackup::write_restore_script(out_dir, &backups) {
                Ok(path) => tracing::info!(script = %path.display(), "wrote the rollback script"),
                Err(e) => tracing::warn!(error = %e, "could not write the rollback script"),
            }
        }
        Ok(backups)
    }

    // -- Uninstall flow ------------------------------------------------------

    fn kill_processes(&self, executables: &[String]) -> Result<Vec<KilledProcess>> {
        let killed = process::kill_matching(executables)?;
        if !killed.is_empty() {
            // Give the kernel a moment to close file handles, or the
            // uninstaller trips over files that are "still in use".
            process::wait_until_gone(executables, PROCESS_EXIT_GRACE);
        }
        Ok(killed)
    }

    fn stop_services(&self, service_names: &[String]) -> Result<StepResult> {
        let mut stopped = Vec::new();
        let mut skipped = Vec::new();
        let mut failed = Vec::new();

        for name in service_names {
            match services::stop(name) {
                Ok(result) if result.status == cwico_core::StepStatus::Skipped => {
                    skipped.push(name.clone())
                }
                Ok(_) => stopped.push(name.clone()),
                Err(e) => failed.push(format!("{name}: {e}")),
            }
        }

        if !failed.is_empty() {
            return Err(cwico_core::Error::Service {
                service: service_names.join(", "),
                source_msg: failed.join("; "),
            });
        }
        Ok(if stopped.is_empty() {
            StepResult::skipped(format!(
                "no running service to stop ({} already stopped or absent)",
                skipped.len()
            ))
        } else {
            StepResult::ok(format!("stopped {} service(s)", stopped.len())).with_artifacts(stopped)
        })
    }

    fn set_service_startup(&self, service: &str, enabled: bool) -> Result<StepResult> {
        let start = if enabled {
            services::StartType::Automatic
        } else {
            services::StartType::Disabled
        };
        services::set_start_type(service, start)
    }

    fn set_task_enabled(&self, task: &str, enabled: bool) -> Result<StepResult> {
        tasks::set_enabled(task, enabled)
    }

    fn run_uninstaller(&self, command: &str, silent: bool) -> Result<ExecOutcome> {
        // `silent` is true when the registry supplied a QuietUninstallString,
        // which is already silent; otherwise ask exec to infer the switches
        // where the installer family is known.
        exec::run(command, !silent, exec::DEFAULT_TIMEOUT)
    }

    fn remove_appx_package(&self, package_full_name: &str, all_users: bool) -> Result<StepResult> {
        // RemoveForAllUsers needs elevation; without it, silently downgrading
        // to a per-user removal would leave the package installed for other
        // accounts while reporting success.
        appx::remove_package(package_full_name, all_users && self.elevated)
    }

    fn remove_appx_provisioned(&self, package_name: &str) -> Result<StepResult> {
        if !self.elevated {
            return Ok(StepResult::skipped(
                "deprovisioning needs Administrator rights; the package will return for \
                 new user accounts"
                    .to_string(),
            ));
        }
        appx::deprovision(package_name)
    }

    fn remove_startup_entry(&self, location: &str, name: &str) -> Result<StepResult> {
        startup::remove(location, name)
    }

    // -- Deep clean ----------------------------------------------------------

    fn delete_paths(&self, paths: &[String], dry_run: bool) -> Result<CleanSummary> {
        deepclean::delete_paths(paths, dry_run)
    }

    fn delete_registry(
        &self,
        keys: &[String],
        values: &[String],
        dry_run: bool,
    ) -> Result<CleanSummary> {
        deepclean::delete_registry(keys, values, dry_run)
    }

    fn expand_path(&self, raw: &str) -> String {
        env::expand_path(raw)
    }
}
