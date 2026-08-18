//! Windows backend for tsudev-cwico.
//!
//! Implements [`cwico_core::PlatformBackend`] with direct Win32 and WinRT
//! calls: registry enumeration, the AppX packaging API, the service control
//! manager, the Task Scheduler COM interfaces, `SRSetRestorePointW` and a
//! guarded deep-clean pass.
//!
//! The crate compiles on non-Windows hosts as an empty shell so the workspace
//! stays buildable everywhere; every module that touches the OS is behind
//! `/// Environment-variable expansion. Available on every host so the path
/// guard's inputs can be tested off Windows.
pub mod env;

/// Parsing `UninstallString` and choosing silent switches.
pub mod cmdline;

/// Name and identifier transforms between Windows' several naming schemes.
pub mod naming;

/// The process-termination deny-list.
pub mod protected;

#[cfg(windows)]
pub mod appx;
#[cfg(windows)]
pub mod backend;
#[cfg(windows)]
pub mod deepclean;
#[cfg(windows)]
pub mod elevation;
#[cfg(windows)]
pub mod exec;
#[cfg(windows)]
pub mod process;
#[cfg(windows)]
pub mod regbackup;
#[cfg(windows)]
pub mod registry;
#[cfg(windows)]
pub mod restore;
#[cfg(windows)]
pub mod scanner;
#[cfg(windows)]
pub mod services;
#[cfg(windows)]
pub mod startup;
#[cfg(windows)]
pub mod tasks;
#[cfg(windows)]
pub mod tweak_apply;
#[cfg(windows)]
pub mod wide;

#[cfg(windows)]
pub use backend::WindowsBackend;
