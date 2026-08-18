//! Error taxonomy shared by every layer of tsudev-cwico.
//!
//! The engine never panics on a per-item failure: an item that cannot be
//! removed records its [`Error`] in the step outcome and the run continues.
//! Only misconfiguration (a malformed safety database, a missing backup
//! directory) aborts a whole operation.

use std::path::PathBuf;

/// Result alias used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The operation requires an elevated (Administrator) process.
    #[error("operation requires Administrator privileges: {0}")]
    NeedsElevation(String),

    /// The item is classified `Critical` and removal is refused outright.
    #[error("refusing to remove protected system component: {name} ({reason})")]
    ProtectedComponent { name: String, reason: String },

    /// The caller asked for an item that is not in the current scan report.
    #[error("unknown item id: {0}")]
    UnknownItem(String),

    /// A safety precondition (restore point, registry backup) could not be met
    /// and the run was configured to require it.
    #[error("safety precondition failed: {0}")]
    SafetyPrecondition(String),

    /// Creating the System Restore Point failed.
    #[error("could not create system restore point: {0}")]
    RestorePoint(String),

    /// Exporting a registry key to a `.reg` backup failed.
    #[error("registry backup failed for {key}: {source_msg}")]
    RegistryBackup { key: String, source_msg: String },

    /// A registry read/write/delete failed.
    #[error("registry operation failed on {key}: {source_msg}")]
    Registry { key: String, source_msg: String },

    /// Launching or waiting on an external uninstaller failed.
    #[error("uninstaller process failed ({command}): {source_msg}")]
    UninstallerProcess { command: String, source_msg: String },

    /// Terminating a running process failed.
    #[error("could not terminate process {pid} ({name}): {source_msg}")]
    ProcessKill {
        pid: u32,
        name: String,
        source_msg: String,
    },

    /// An AppX / MSIX package operation failed.
    #[error("appx operation failed for {package}: {source_msg}")]
    Appx { package: String, source_msg: String },

    /// A Windows service could not be queried or reconfigured.
    #[error("service operation failed for {service}: {source_msg}")]
    Service { service: String, source_msg: String },

    /// A scheduled task could not be queried or reconfigured.
    #[error("scheduled task operation failed for {task}: {source_msg}")]
    ScheduledTask { task: String, source_msg: String },

    /// Deep-clean refused to touch a path because it failed the guard rails.
    #[error("unsafe delete target rejected: {path} ({reason})")]
    UnsafeDeleteTarget { path: PathBuf, reason: String },

    /// The safety database could not be read or parsed.
    #[error("safety database error: {0}")]
    SafetyDatabase(String),

    /// The current platform has no implementation for this call.
    #[error("not supported on this platform: {0}")]
    Unsupported(String),

    #[error("i/o error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Escape hatch for platform errors with no better mapping.
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// Attach a path to a bare [`std::io::Error`].
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }

    /// `true` when retrying the same operation could plausibly succeed
    /// (a locked file, a process that had not exited yet). The engine uses
    /// this to decide whether a second pass is worth scheduling.
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Error::ProcessKill { .. } | Error::UninstallerProcess { .. } | Error::Io { .. }
        )
    }

    /// A stable machine-readable code, surfaced to the UI for translation.
    pub fn code(&self) -> &'static str {
        match self {
            Error::NeedsElevation(_) => "needs_elevation",
            Error::ProtectedComponent { .. } => "protected_component",
            Error::UnknownItem(_) => "unknown_item",
            Error::SafetyPrecondition(_) => "safety_precondition",
            Error::RestorePoint(_) => "restore_point",
            Error::RegistryBackup { .. } => "registry_backup",
            Error::Registry { .. } => "registry",
            Error::UninstallerProcess { .. } => "uninstaller_process",
            Error::ProcessKill { .. } => "process_kill",
            Error::Appx { .. } => "appx",
            Error::Service { .. } => "service",
            Error::ScheduledTask { .. } => "scheduled_task",
            Error::UnsafeDeleteTarget { .. } => "unsafe_delete_target",
            Error::SafetyDatabase(_) => "safety_database",
            Error::Unsupported(_) => "unsupported",
            Error::Io { .. } => "io",
            Error::Json(_) => "json",
            Error::Other(_) => "other",
        }
    }
}
