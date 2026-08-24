//! The executor: walks a validated [`RemovalPlan`] and drives the backend.
//!
//! Failure policy: an individual step failing records the error and moves on
//! to the next *item*; the remaining steps of the failed item are skipped,
//! because running a deep clean after a failed uninstall is how you end up
//! with a half-removed product. The preamble is different - if the restore
//! point was required and could not be created, the run does not start at all.

use crate::backend::{
    CleanSummary, Event, EventSink, LogLevel, PlatformBackend, RegistryBackup, RestorePointInfo,
    StepResult, StepStatus,
};
use crate::error::{Error, Result};
use crate::plan::{RemovalPlan, Step};
use crate::safety::SafetyDatabase;
use crate::scan::{ScanOptions, ScanReport};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// Result of one step, as recorded in the run report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepOutcome {
    pub step: String,
    pub status: StepStatus,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_freed: Option<u64>,
}

/// Result of one item's worth of work.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemOutcome {
    pub item_id: String,
    pub name: String,
    pub status: StepStatus,
    pub steps: Vec<StepOutcome>,
    pub bytes_freed: u64,
}

/// Everything that happened during a run. Written to disk as the transaction
/// log so a later session - or a support engineer - can see exactly what was
/// changed and which `.reg` files restore it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunReport {
    pub started_at: String,
    pub finished_at: String,
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restore_point: Option<RestorePointInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub registry_backups: Vec<RegistryBackup>,
    pub items: Vec<ItemOutcome>,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub bytes_freed: u64,
    /// `true` when any uninstaller returned 3010.
    pub reboot_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_log: Option<PathBuf>,
    /// Preamble problems that did not abort the run (e.g. registry backup
    /// failed for one key while the rest succeeded).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

impl RunReport {
    pub fn all_succeeded(&self) -> bool {
        self.failed == 0
    }
}

/// Drives a backend through scans and plans.
pub struct Engine<'a> {
    backend: &'a dyn PlatformBackend,
    db: &'a SafetyDatabase,
}

impl std::fmt::Debug for Engine<'_> {
    /// `PlatformBackend` is a trait object, so report the facts that identify
    /// this engine instead of trying to format the backend itself.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let info = self.backend.platform_info();
        f.debug_struct("Engine")
            .field("platform", &info.platform)
            .field("elevated", &info.elevated)
            .field("safety_db", &self.db)
            .finish()
    }
}

impl<'a> Engine<'a> {
    pub fn new(backend: &'a dyn PlatformBackend, db: &'a SafetyDatabase) -> Self {
        Self { backend, db }
    }

    pub fn backend(&self) -> &dyn PlatformBackend {
        self.backend
    }

    pub fn safety_db(&self) -> &SafetyDatabase {
        self.db
    }

    pub fn scan(&self, options: &ScanOptions, sink: &dyn EventSink) -> Result<ScanReport> {
        let started = Instant::now();
        sink.emit(Event::ScanStarted {
            passes: options.enabled_pass_count(),
        });
        let mut report = self.backend.scan(options, self.db, sink)?;
        report.finalize(started.elapsed().as_millis() as u64);
        sink.emit(Event::ScanFinished {
            total: report.stats.total,
            duration_ms: report.stats.duration_ms,
        });
        Ok(report)
    }

    /// Execute a plan.
    ///
    /// Returns `Err` only for run-level aborts: a `Critical` item that somehow
    /// reached execution, a missing elevation, or a required restore point
    /// that could not be created. Per-item failures live in the report.
    pub fn execute(&self, plan: &RemovalPlan, sink: &dyn EventSink) -> Result<RunReport> {
        plan.assert_no_protected_items()?;

        let started_at = now_rfc3339();
        let run_start = Instant::now();
        let total_steps = plan.total_steps();

        if !plan.options.dry_run && !self.backend.is_elevated() {
            return Err(Error::NeedsElevation(
                "removing software requires running tsudev-cwico as Administrator".into(),
            ));
        }

        sink.emit(Event::RunStarted {
            total_steps,
            dry_run: plan.options.dry_run,
        });

        let mut report = RunReport {
            started_at: started_at.clone(),
            finished_at: String::new(),
            dry_run: plan.options.dry_run,
            restore_point: None,
            registry_backups: Vec::new(),
            items: Vec::new(),
            succeeded: 0,
            failed: 0,
            skipped: 0,
            bytes_freed: 0,
            reboot_required: false,
            transaction_log: None,
            warnings: Vec::new(),
        };

        let mut step_index = 0usize;

        // ---- Preamble: restore point, then registry backup -----------------
        for step in &plan.preamble {
            step_index += 1;
            sink.emit(Event::StepStarted {
                item_id: None,
                step: step.slug().to_string(),
                index: step_index,
                total: total_steps,
            });

            match step {
                Step::CreateRestorePoint { description } => {
                    match self.backend.create_restore_point(description) {
                        Ok(info) => {
                            sink.emit(Event::StepFinished {
                                item_id: None,
                                step: step.slug().into(),
                                status: StepStatus::Succeeded,
                                detail: format!("restore point #{} created", info.sequence_number),
                            });
                            report.restore_point = Some(info);
                        }
                        Err(e) if plan.options.require_restore_point => {
                            sink.emit(Event::StepFinished {
                                item_id: None,
                                step: step.slug().into(),
                                status: StepStatus::Failed,
                                detail: e.to_string(),
                            });
                            // A rollback you cannot perform is not a rollback.
                            return Err(Error::SafetyPrecondition(format!(
                                "the run was cancelled because no restore point could be \
                                 created: {e}. Enable System Protection on the system drive, \
                                 or turn off 'require a restore point' to proceed without one."
                            )));
                        }
                        Err(e) => {
                            let msg = format!("continuing without a restore point: {e}");
                            sink.emit(Event::StepFinished {
                                item_id: None,
                                step: step.slug().into(),
                                status: StepStatus::Failed,
                                detail: msg.clone(),
                            });
                            report.warnings.push(msg);
                        }
                    }
                }

                Step::BackupRegistry { keys } => {
                    let dir = plan
                        .options
                        .backup_dir
                        .clone()
                        .unwrap_or_else(|| PathBuf::from("."));
                    match self.backend.backup_registry_keys(keys, &dir) {
                        Ok(backups) => {
                            sink.emit(Event::StepFinished {
                                item_id: None,
                                step: step.slug().into(),
                                status: StepStatus::Succeeded,
                                detail: format!("{} registry key(s) exported", backups.len()),
                            });
                            report.registry_backups = backups;
                        }
                        Err(e) => {
                            let msg = format!("registry backup incomplete: {e}");
                            sink.emit(Event::StepFinished {
                                item_id: None,
                                step: step.slug().into(),
                                status: StepStatus::Failed,
                                detail: msg.clone(),
                            });
                            report.warnings.push(msg);
                        }
                    }
                }

                other => {
                    report
                        .warnings
                        .push(format!("unexpected preamble step {}", other.slug()));
                }
            }
        }

        // ---- Items ---------------------------------------------------------
        'items: for planned in &plan.items {
            let mut outcome = ItemOutcome {
                item_id: planned.item_id.clone(),
                name: planned.name.clone(),
                status: StepStatus::Succeeded,
                steps: Vec::new(),
                bytes_freed: 0,
            };

            for step in &planned.steps {
                step_index += 1;
                sink.emit(Event::StepStarted {
                    item_id: Some(planned.item_id.clone()),
                    step: step.slug().to_string(),
                    index: step_index,
                    total: total_steps,
                });

                let step_start = Instant::now();
                let result = self.run_step(step, plan.options.dry_run);
                let duration_ms = step_start.elapsed().as_millis() as u64;

                let (status, detail, artifacts, error_code, bytes) = match result {
                    Ok(r) => (r.status, r.detail, r.artifacts, None, r.bytes_freed),
                    Err(e) => (
                        StepStatus::Failed,
                        e.to_string(),
                        Vec::new(),
                        Some(e.code().to_string()),
                        None,
                    ),
                };

                if let Some(b) = bytes {
                    outcome.bytes_freed = outcome.bytes_freed.saturating_add(b);
                }
                if detail.contains("3010") {
                    report.reboot_required = true;
                }

                sink.emit(Event::StepFinished {
                    item_id: Some(planned.item_id.clone()),
                    step: step.slug().to_string(),
                    status,
                    detail: detail.clone(),
                });

                let failed = status.is_failure();
                outcome.steps.push(StepOutcome {
                    step: step.slug().to_string(),
                    status,
                    detail,
                    artifacts,
                    error_code,
                    duration_ms,
                    bytes_freed: bytes,
                });

                if failed {
                    // Stop this item here: the later steps assume the earlier
                    // ones worked.
                    outcome.status = StepStatus::Failed;
                    sink.log(
                        LogLevel::Warn,
                        format!(
                            "`{}` failed at step `{}`; remaining steps for this item skipped",
                            planned.name,
                            step.slug()
                        ),
                    );
                    break;
                }
            }

            if outcome.status != StepStatus::Failed {
                // An item whose every step was a no-op is "skipped", not
                // "succeeded" - it was already gone.
                outcome.status = if outcome
                    .steps
                    .iter()
                    .all(|s| matches!(s.status, StepStatus::Skipped))
                    && !outcome.steps.is_empty()
                {
                    StepStatus::Skipped
                } else if plan.options.dry_run {
                    StepStatus::Simulated
                } else {
                    StepStatus::Succeeded
                };
            }

            match outcome.status {
                StepStatus::Failed => report.failed += 1,
                StepStatus::Skipped => report.skipped += 1,
                _ => report.succeeded += 1,
            }
            report.bytes_freed = report.bytes_freed.saturating_add(outcome.bytes_freed);

            sink.emit(Event::ItemFinished {
                item_id: outcome.item_id.clone(),
                name: outcome.name.clone(),
                status: outcome.status,
            });

            let stop_now = outcome.status.is_failure() && !plan.options.continue_on_error;
            report.items.push(outcome);
            if stop_now {
                sink.log(
                    LogLevel::Error,
                    "stopping the run after a failure (continue-on-error is off)".into(),
                );
                break 'items;
            }
        }

        report.finished_at = now_rfc3339();
        let duration_ms = run_start.elapsed().as_millis() as u64;
        sink.emit(Event::RunFinished {
            succeeded: report.succeeded,
            failed: report.failed,
            skipped: report.skipped,
            duration_ms,
        });

        // ---- Transaction log ------------------------------------------------
        if let Some(dir) = &plan.options.backup_dir {
            if !plan.options.dry_run {
                match self.write_transaction_log(dir, &report) {
                    Ok(path) => report.transaction_log = Some(path),
                    Err(e) => report
                        .warnings
                        .push(format!("could not write the transaction log: {e}")),
                }
            }
        }

        Ok(report)
    }

    fn run_step(&self, step: &Step, dry_run: bool) -> Result<StepResult> {
        match step {
            Step::CreateRestorePoint { .. } | Step::BackupRegistry { .. } => {
                Ok(StepResult::skipped("handled in the run preamble"))
            }

            Step::KillProcesses { executables } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!(
                        "would terminate processes matching: {}",
                        executables.join(", ")
                    )));
                }
                let killed = self.backend.kill_processes(executables)?;
                if killed.is_empty() {
                    Ok(StepResult::skipped("no matching process was running"))
                } else {
                    let names: Vec<String> = killed
                        .iter()
                        .map(|p| format!("{} (pid {})", p.name, p.pid))
                        .collect();
                    Ok(
                        StepResult::ok(format!("terminated {} process(es)", killed.len()))
                            .with_artifacts(names),
                    )
                }
            }

            Step::StopServices { services } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!(
                        "would stop: {}",
                        services.join(", ")
                    )));
                }
                self.backend.stop_services(services)
            }

            Step::DisableTasks { tasks } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!(
                        "would disable {} scheduled task(s)",
                        tasks.len()
                    )));
                }
                let mut disabled = 0;
                let mut artifacts = Vec::new();
                for task in tasks {
                    match self.backend.set_task_enabled(task, false) {
                        Ok(r) if r.status == StepStatus::Succeeded => {
                            disabled += 1;
                            artifacts.push(task.clone());
                        }
                        Ok(_) => {}
                        Err(e) => artifacts.push(format!("{task}: {e}")),
                    }
                }
                if disabled == 0 {
                    Ok(StepResult::skipped("no matching scheduled task"))
                } else {
                    Ok(
                        StepResult::ok(format!("disabled {disabled} scheduled task(s)"))
                            .with_artifacts(artifacts),
                    )
                }
            }

            Step::RunOfficialUninstaller { command, silent } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!("would run: {command}")));
                }
                let outcome = self.backend.run_uninstaller(command, *silent)?;
                if outcome.is_success() {
                    let mut detail = format!(
                        "uninstaller exited with {}",
                        outcome
                            .exit_code
                            .map(|c| c.to_string())
                            .unwrap_or_else(|| "no exit code".into())
                    );
                    if outcome.needs_reboot() {
                        detail.push_str(" (3010: a restart is required to finish)");
                    }
                    Ok(StepResult::ok(detail))
                } else {
                    Err(Error::UninstallerProcess {
                        command: command.clone(),
                        source_msg: format!(
                            "exit code {:?}{}{}",
                            outcome.exit_code,
                            if outcome.timed_out { ", timed out" } else { "" },
                            if outcome.stderr_tail.is_empty() {
                                String::new()
                            } else {
                                format!(": {}", outcome.stderr_tail)
                            }
                        ),
                    })
                }
            }

            Step::RemoveAppxPackage {
                package_full_name,
                all_users,
            } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!(
                        "would remove AppX package {package_full_name}{}",
                        if *all_users { " for all users" } else { "" }
                    )));
                }
                self.backend
                    .remove_appx_package(package_full_name, *all_users)
            }

            Step::RemoveAppxProvisioned { package_name } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!(
                        "would deprovision {package_name}"
                    )));
                }
                self.backend.remove_appx_provisioned(package_name)
            }

            Step::DeepCleanFiles { paths } => {
                let summary = self.backend.delete_paths(paths, dry_run)?;
                Ok(Self::summarize_clean(summary, dry_run, "folder"))
            }

            Step::DeepCleanRegistry { keys, values } => {
                let summary = self.backend.delete_registry(keys, values, dry_run)?;
                Ok(Self::summarize_clean(summary, dry_run, "registry entry"))
            }

            Step::SetServiceStartup { service, enabled } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!(
                        "would set `{service}` start type to {}",
                        if *enabled { "automatic" } else { "disabled" }
                    )));
                }
                self.backend.set_service_startup(service, *enabled)
            }

            Step::SetTaskEnabled { task, enabled } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!(
                        "would {} task `{task}`",
                        if *enabled { "enable" } else { "disable" }
                    )));
                }
                self.backend.set_task_enabled(task, *enabled)
            }

            Step::RemoveStartupEntry { location, name } => {
                if dry_run {
                    return Ok(StepResult::simulated(format!(
                        "would remove startup entry `{name}` from {location}"
                    )));
                }
                self.backend.remove_startup_entry(location, name)
            }
        }
    }

    fn summarize_clean(summary: CleanSummary, dry_run: bool, noun: &str) -> StepResult {
        let status = if !summary.failed.is_empty() {
            StepStatus::Failed
        } else if dry_run {
            StepStatus::Simulated
        } else if summary.removed.is_empty() {
            StepStatus::Skipped
        } else {
            StepStatus::Succeeded
        };
        let detail = if summary.removed.is_empty() && summary.failed.is_empty() {
            format!("no {noun} residue found")
        } else {
            summary.describe()
        };
        let mut artifacts = summary.removed.clone();
        artifacts.extend(summary.failed.iter().map(|f| format!("FAILED: {f}")));
        StepResult {
            status,
            detail,
            artifacts,
            bytes_freed: Some(summary.bytes_freed),
        }
    }

    fn write_transaction_log(&self, dir: &std::path::Path, report: &RunReport) -> Result<PathBuf> {
        std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
        let path = dir.join(format!(
            "cwico-run-{}.json",
            report.started_at.replace([':', '.'], "-")
        ));
        let json = serde_json::to_string_pretty(report)?;
        std::fs::write(&path, json).map_err(|e| Error::io(&path, e))?;
        Ok(path)
    }
}

/// RFC 3339 timestamp for logs and file names.
pub fn now_rfc3339() -> String {
    use time::format_description::well_known::Rfc3339;
    time::OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}
