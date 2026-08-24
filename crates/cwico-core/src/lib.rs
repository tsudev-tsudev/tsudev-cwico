//! # cwico-core
//!
//! The platform-independent half of **tsudev-cwico**, a deep Windows
//! debloater and software-removal toolkit.
//!
//! Nothing in this crate touches an operating system. Scanning, classifying
//! and planning happen here; execution happens behind
//! [`backend::PlatformBackend`], which `cwico-win` implements with Win32 and
//! WinRT calls. That split means the dangerous logic - what counts as safe to
//! remove, which paths may be deleted, which items are hard-blocked - is
//! ordinary testable Rust that runs on any host.
//!
//! ## The safety model
//!
//! Three things stand between a user and a broken machine, in order:
//!
//! 1. [`safety::SafetyDatabase`] classifies every discovered item. Unmatched
//!    items are [`model::SafetyClass::Unknown`], never `Safe`.
//! 2. [`plan::RemovalPlan::build`] refuses to plan a `Critical` item at all,
//!    and requires an explicit per-item confirmation for `Caution`/`Unknown`.
//! 3. [`guard`] validates every filesystem path and registry key immediately
//!    before deletion, rejecting drive roots, system directories, user data
//!    folders and hive roots.
//!
//! On top of that, [`engine::Engine::execute`] takes a System Restore Point
//! and exports a `.reg` backup of every key it is about to touch, and writes
//! a transaction log describing exactly what changed.
//!
//! ## Typical use
//!
//! ```no_run
//! use cwico_core::{Engine, SafetyDatabase, ScanOptions, RemovalPlan, PlanOptions, Selection};
//! use cwico_core::backend::NullSink;
//! # fn demo(backend: &dyn cwico_core::backend::PlatformBackend) -> cwico_core::Result<()> {
//! let db = SafetyDatabase::builtin();
//! let engine = Engine::new(backend, &db);
//!
//! let report = engine.scan(&ScanOptions::default(), &NullSink)?;
//! let picks: Vec<Selection> = report
//!     .bulk_selectable()
//!     .iter()
//!     .map(|item| Selection::uninstall(&item.id))
//!     .collect();
//!
//! let plan = RemovalPlan::build(&report, &picks, PlanOptions::dry_run());
//! let run = engine.execute(&plan, &NullSink)?;
//! println!("{} succeeded, {} failed", run.succeeded, run.failed);
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_debug_implementations)]

pub mod backend;
pub mod engine;
pub mod error;
pub mod guard;
pub mod model;
pub mod plan;
pub mod safety;
pub mod scan;
pub mod tweaks;
pub mod version;

#[cfg(feature = "mock")]
pub mod mock;

pub use backend::{Event, EventSink, NullSink, PlatformBackend, PlatformInfo, StepStatus};
pub use engine::{Engine, ItemOutcome, RunReport, StepOutcome};
pub use error::{Error, Result};
pub use model::{
    Action, Architecture, InstallScope, ItemState, Locale, LocalizedText, SafetyClass,
    SoftwareItem, SourceKind,
};
pub use plan::{PlanOptions, PlannedItem, RejectedSelection, RemovalPlan, Selection, Step};
pub use safety::{SafetyDatabase, SafetyRule, Verdict};
pub use scan::{ItemFilter, ScanOptions, ScanReport, ScanStats, SortBy};
pub use tweaks::{Tweak, TweakCatalog, TweakCategory};
pub use version::{current_release_name, name_for_semver, Release};

/// Product name, used in restore-point descriptions and log headers.
pub const PRODUCT_NAME: &str = "tsudev-cwico";

/// Product homepage. The UI links its logo and wordmark here.
pub const PRODUCT_URL: &str = "https://tsudev.com";

/// Crate version, mirroring the workspace version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
