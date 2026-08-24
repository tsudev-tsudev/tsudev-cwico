//! Update checking and installation.
//!
//! ## Why this is a blocking gate rather than a notification
//!
//! The safety database decides what this tool will and will not remove. When a
//! rule turns out to be wrong - something classified `Safe` that costs a user
//! a feature, or worse, something that should have been `Critical` - the fix
//! ships as a new version. A user running last month's build is running last
//! month's idea of what is safe to delete on their machine.
//!
//! For an ordinary application an ignorable "update available" banner is the
//! right call. For one that deletes system components with Administrator
//! rights, it is not.
//!
//! ## Why it nonetheless fails open
//!
//! The gate closes only when a newer version is *confirmed*. A network error,
//! a DNS failure or a GitHub outage leaves the app running normally with a
//! quiet note in the UI.
//!
//! The reasoning: a mandatory update protects users from a stale safety
//! database, which is a slow, bounded risk. A check that fails closed turns
//! any outage into every user losing the tool at once - including the user who
//! needs it right now to undo something. Trading a bounded risk for an
//! unbounded one is the wrong way round.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// The channel install progress is emitted on.
pub const PROGRESS_EVENT: &str = "cwico://update-progress";

/// What the front end needs to decide whether to show the gate.
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    /// `true` only when a newer version was confirmed. This is the one field
    /// that closes the gate.
    pub available: bool,
    /// `false` when the check itself could not complete - offline, DNS,
    /// GitHub down. The app runs normally; the UI says so quietly.
    pub checked: bool,
    /// Why the check failed, when it did. Shown in the details pane, not as
    /// an error dialog: a failed update check is not the user's problem.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_error: Option<String>,

    /// The running build, as a release name (`tsudev-cwico-v26.8.19`).
    pub current_release: String,
    /// …and as the raw semver, for support reports.
    pub current_version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_release: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_version: Option<String>,
    /// Release notes from `latest.json`, if the release had a body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
}

impl UpdateStatus {
    fn current() -> Self {
        Self {
            current_release: cwico_core::current_release_name(),
            current_version: cwico_core::VERSION.to_string(),
            ..Default::default()
        }
    }

    /// The check completed and the build is current.
    pub fn up_to_date() -> Self {
        Self {
            checked: true,
            ..Self::current()
        }
    }

    /// The check could not run. Fail open: `available` stays false.
    pub fn check_failed(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        tracing::warn!(error = %reason, "update check failed; continuing without it");
        Self {
            checked: false,
            check_error: Some(reason),
            ..Self::current()
        }
    }
}

/// Progress of an in-flight download, streamed to the update screen.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub downloaded: u64,
    /// `None` when the server sent no `Content-Length`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// `true` once the download is complete and the installer is running.
    pub installing: bool,
}

// ---------------------------------------------------------------------------
// Windows
// ---------------------------------------------------------------------------

#[cfg(windows)]
pub async fn check(app: &AppHandle) -> UpdateStatus {
    use tauri_plugin_updater::UpdaterExt;

    let updater = match app.updater() {
        Ok(updater) => updater,
        Err(e) => return UpdateStatus::check_failed(format!("updater unavailable: {e}")),
    };

    match updater.check().await {
        Ok(Some(update)) => {
            let new_version = update.version.clone();
            tracing::info!(
                current = %cwico_core::VERSION,
                available = %new_version,
                "a newer release is available"
            );
            UpdateStatus {
                available: true,
                checked: true,
                new_release: Some(cwico_core::name_for_semver(&new_version)),
                new_version: Some(new_version),
                notes: update.body.clone().filter(|b| !b.trim().is_empty()),
                published_at: update.date.map(|d| d.to_string()),
                ..UpdateStatus::current()
            }
        }
        Ok(None) => {
            tracing::info!(version = %cwico_core::VERSION, "this build is current");
            UpdateStatus::up_to_date()
        }
        Err(e) => UpdateStatus::check_failed(e.to_string()),
    }
}

/// Download and install the update, then restart into it.
///
/// Returns only on failure: on success the process is replaced by the new
/// version.
#[cfg(windows)]
pub async fn install(app: &AppHandle) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;

    let updater = app
        .updater()
        .map_err(|e| format!("updater unavailable: {e}"))?;

    // Re-checked rather than carried over from `check`: the handle is not
    // cheaply storable across commands, and a second round trip costs less
    // than the state management would.
    let update = updater
        .check()
        .await
        .map_err(|e| format!("could not reach the update server: {e}"))?
        .ok_or_else(|| "there is no update to install".to_string())?;

    tracing::info!(version = %update.version, "downloading update");

    let emitter = app.clone();
    let mut downloaded: u64 = 0;

    update
        .download_and_install(
            move |chunk: usize, total: Option<u64>| {
                downloaded += chunk as u64;
                let _ = emitter.emit(
                    PROGRESS_EVENT,
                    UpdateProgress {
                        downloaded,
                        total,
                        installing: false,
                    },
                );
            },
            || {
                tracing::info!("download complete; running the installer");
            },
        )
        .await
        .map_err(|e| format!("the update could not be installed: {e}"))?;

    let _ = app.emit(
        PROGRESS_EVENT,
        UpdateProgress {
            downloaded: 0,
            total: None,
            installing: true,
        },
    );

    tracing::info!("update installed; restarting");
    app.restart();
}

// ---------------------------------------------------------------------------
// Everywhere else
// ---------------------------------------------------------------------------

/// A non-Windows build has no installer to replace itself with, so it reports
/// itself current rather than pretending to check.
#[cfg(not(windows))]
pub async fn check(_app: &AppHandle) -> UpdateStatus {
    UpdateStatus::up_to_date()
}

#[cfg(not(windows))]
pub async fn install(_app: &AppHandle) -> Result<(), String> {
    Err("updates are only delivered to Windows builds".into())
}
