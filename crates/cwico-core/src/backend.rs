//! The seam between the platform-independent engine and the operating system.
//!
//! Everything that actually touches Windows lives behind [`PlatformBackend`].
//! `cwico-win` implements it with Win32/WinRT calls; `MockBackend` implements
//! it in memory so the planner, the safety gate and the whole UI can be
//! exercised on a Linux CI runner.

use crate::error::Result;
use crate::model::SoftwareItem;
use crate::safety::SafetyDatabase;
use crate::scan::{ScanOptions, ScanReport};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Progress events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Streamed to the UI while a scan or a run is in flight.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Event {
    ScanStarted {
        passes: usize,
    },
    ScanPassStarted {
        pass: String,
        index: usize,
        total: usize,
    },
    ScanPassFinished {
        pass: String,
        found: usize,
    },
    ScanFinished {
        total: usize,
        duration_ms: u64,
    },
    RunStarted {
        total_steps: usize,
        dry_run: bool,
    },
    StepStarted {
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        step: String,
        index: usize,
        total: usize,
    },
    StepFinished {
        #[serde(skip_serializing_if = "Option::is_none")]
        item_id: Option<String>,
        step: String,
        status: StepStatus,
        detail: String,
    },
    ItemFinished {
        item_id: String,
        name: String,
        status: StepStatus,
    },
    RunFinished {
        succeeded: usize,
        failed: usize,
        skipped: usize,
        duration_ms: u64,
    },
    Log {
        level: LogLevel,
        message: String,
    },
}

/// Where progress events go. The GUI forwards them over Tauri's event bus;
/// the CLI prints them; tests collect them into a `Vec`.
pub trait EventSink: Send + Sync {
    fn emit(&self, event: Event);

    /// Convenience for a free-text log line. Takes `String` rather than
    /// `impl Into<String>` so the trait stays object-safe: the engine only
    /// ever holds a `&dyn EventSink`.
    fn log(&self, level: LogLevel, message: String) {
        self.emit(Event::Log { level, message });
    }
}

/// Discards everything. Useful for tests and for non-interactive runs.
#[derive(Debug)]
pub struct NullSink;

impl EventSink for NullSink {
    fn emit(&self, _event: Event) {}
}

impl<T: EventSink + ?Sized> EventSink for std::sync::Arc<T> {
    fn emit(&self, event: Event) {
        (**self).emit(event);
    }
}

// ---------------------------------------------------------------------------
// Step results
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    Succeeded,
    /// Nothing to do - the process was not running, the key did not exist.
    /// Not a failure, and reported separately so the log is honest.
    Skipped,
    Failed,
    /// `dry_run` was set: the step reports what it would have done.
    Simulated,
}

impl StepStatus {
    pub fn is_failure(self) -> bool {
        matches!(self, StepStatus::Failed)
    }
}

/// Result of one backend call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepResult {
    pub status: StepStatus,
    /// Human-readable summary: "terminated 3 processes", "key not present".
    pub detail: String,
    /// Files/keys/PIDs the step touched, for the audit log.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
    /// Bytes reclaimed, when the step deleted files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_freed: Option<u64>,
}

impl StepResult {
    pub fn ok(detail: impl Into<String>) -> Self {
        Self {
            status: StepStatus::Succeeded,
            detail: detail.into(),
            artifacts: Vec::new(),
            bytes_freed: None,
        }
    }

    pub fn skipped(detail: impl Into<String>) -> Self {
        Self {
            status: StepStatus::Skipped,
            detail: detail.into(),
            artifacts: Vec::new(),
            bytes_freed: None,
        }
    }

    pub fn simulated(detail: impl Into<String>) -> Self {
        Self {
            status: StepStatus::Simulated,
            detail: detail.into(),
            artifacts: Vec::new(),
            bytes_freed: None,
        }
    }

    pub fn with_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn with_bytes(mut self, bytes: u64) -> Self {
        self.bytes_freed = Some(bytes);
        self
    }
}

// ---------------------------------------------------------------------------
// Backend payload types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    /// `windows`, `linux`, `mock`.
    pub platform: String,
    /// "Windows 11 Pro 23H2 (build 22631.3007)".
    pub os_description: String,
    pub os_build: Option<String>,
    pub arch: String,
    pub elevated: bool,
    /// `false` when System Restore is turned off for the system drive, so the
    /// UI can offer to enable it before a destructive run.
    pub system_restore_available: bool,
}

/// A restore point the tool created.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePointInfo {
    pub sequence_number: i64,
    pub description: String,
    pub created_at: String,
}

/// One exported `.reg` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryBackup {
    pub key: String,
    pub file: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KilledProcess {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
}

/// Outcome of running an external uninstaller.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecOutcome {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout_tail: String,
    pub stderr_tail: String,
    pub timed_out: bool,
    pub duration_ms: u64,
}

impl ExecOutcome {
    /// MSI and most installers use 0; 3010 means "success, reboot required";
    /// 1605 means "this product is not installed", which is a no-op success
    /// for our purposes.
    pub fn is_success(&self) -> bool {
        matches!(self.exit_code, Some(0) | Some(3010) | Some(1605))
    }

    pub fn needs_reboot(&self) -> bool {
        self.exit_code == Some(3010)
    }
}

/// What a deep-clean pass removed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanSummary {
    pub removed: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
    pub bytes_freed: u64,
}

impl CleanSummary {
    pub fn describe(&self) -> String {
        format!(
            "{} removed, {} skipped, {} failed",
            self.removed.len(),
            self.skipped.len(),
            self.failed.len()
        )
    }
}

// ---------------------------------------------------------------------------
// The trait
// ---------------------------------------------------------------------------

/// Every operating-system interaction the engine needs.
///
/// Implementations must be **idempotent** and must treat "already gone" as
/// [`StepStatus::Skipped`], never as an error: a user who runs the same plan
/// twice should see a clean second run, not a wall of failures.
pub trait PlatformBackend: Send + Sync {
    fn platform_info(&self) -> PlatformInfo;

    /// `true` when the process holds Administrator rights.
    fn is_elevated(&self) -> bool {
        self.platform_info().elevated
    }

    /// Sweep the system. Classification against `db` happens inside so that
    /// each pass can classify as it discovers, keeping the UI responsive.
    fn scan(
        &self,
        options: &ScanOptions,
        db: &SafetyDatabase,
        sink: &dyn EventSink,
    ) -> Result<ScanReport>;

    /// Measure or refresh a single item - used by the details pane.
    fn refresh_item(&self, item: &SoftwareItem) -> Result<Option<SoftwareItem>> {
        Ok(Some(item.clone()))
    }

    // -- Safety scaffolding --------------------------------------------------

    fn create_restore_point(&self, description: &str) -> Result<RestorePointInfo>;

    /// Export the given keys to `.reg` files under `out_dir`.
    fn backup_registry_keys(&self, keys: &[String], out_dir: &Path) -> Result<Vec<RegistryBackup>>;

    // -- Uninstall flow ------------------------------------------------------

    /// Terminate every running process whose image name matches one of
    /// `executables`. Matching is case-insensitive on the file name.
    fn kill_processes(&self, executables: &[String]) -> Result<Vec<KilledProcess>>;

    fn stop_services(&self, services: &[String]) -> Result<StepResult>;

    fn set_service_startup(&self, service: &str, enabled: bool) -> Result<StepResult>;

    fn set_task_enabled(&self, task: &str, enabled: bool) -> Result<StepResult>;

    /// Run the vendor's uninstaller. `silent` selects the quiet command line.
    fn run_uninstaller(&self, command: &str, silent: bool) -> Result<ExecOutcome>;

    fn remove_appx_package(&self, package_full_name: &str, all_users: bool) -> Result<StepResult>;

    fn remove_appx_provisioned(&self, package_name: &str) -> Result<StepResult>;

    fn remove_startup_entry(&self, location: &str, name: &str) -> Result<StepResult>;

    // -- Deep clean ----------------------------------------------------------

    /// Delete residual directories. Implementations **must** run the guard
    /// rails in [`crate::guard`] before deleting anything.
    fn delete_paths(&self, paths: &[String], dry_run: bool) -> Result<CleanSummary>;

    fn delete_registry(
        &self,
        keys: &[String],
        values: &[String],
        dry_run: bool,
    ) -> Result<CleanSummary>;

    /// Expand `%LOCALAPPDATA%`-style variables. Exposed so the planner can show
    /// the user real paths in the confirmation dialog.
    fn expand_path(&self, raw: &str) -> String {
        raw.to_string()
    }
}
