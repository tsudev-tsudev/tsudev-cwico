//! Deterministic in-memory backend.
//!
//! Its job is to let the planner, the engine and the entire UI be built and
//! tested away from a Windows machine. The fixture set deliberately contains
//! one item of every safety class and every source kind, so a UI developed
//! against it exercises the confirmation gate and the hard block for real.
//!
//! Only compiled when the `mock` feature is on. Release Windows builds do not
//! include it.

use crate::backend::{
    CleanSummary, Event, EventSink, ExecOutcome, KilledProcess, PlatformBackend, PlatformInfo,
    RegistryBackup, RestorePointInfo, StepResult, StepStatus,
};
use crate::engine::now_rfc3339;
use crate::error::Result;
use crate::guard;
use crate::model::{Architecture, InstallScope, ItemState, SoftwareItem, SourceKind};
use crate::plan::join_list;
use crate::safety::SafetyDatabase;
use crate::scan::{ScanOptions, ScanReport};
use std::path::Path;
use std::sync::Mutex;

/// A record of everything the mock was asked to do, for assertions in tests.
#[derive(Debug, Default, Clone)]
pub struct MockJournal {
    pub restore_points: Vec<String>,
    pub backed_up_keys: Vec<String>,
    pub killed: Vec<String>,
    pub uninstallers_run: Vec<String>,
    pub appx_removed: Vec<String>,
    pub paths_deleted: Vec<String>,
    pub keys_deleted: Vec<String>,
    pub services_changed: Vec<(String, bool)>,
    pub tasks_changed: Vec<(String, bool)>,
}

#[derive(Debug)]
pub struct MockBackend {
    elevated: bool,
    /// Make `create_restore_point` fail, to exercise the abort path.
    restore_point_fails: bool,
    /// Command substrings whose uninstaller should report failure.
    failing_uninstallers: Vec<String>,
    journal: Mutex<MockJournal>,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            elevated: true,
            restore_point_fails: false,
            failing_uninstallers: Vec::new(),
            journal: Mutex::new(MockJournal::default()),
        }
    }

    pub fn unelevated(mut self) -> Self {
        self.elevated = false;
        self
    }

    pub fn with_failing_restore_point(mut self) -> Self {
        self.restore_point_fails = true;
        self
    }

    pub fn with_failing_uninstaller(mut self, needle: impl Into<String>) -> Self {
        self.failing_uninstallers.push(needle.into());
        self
    }

    pub fn journal(&self) -> MockJournal {
        self.journal.lock().expect("journal mutex").clone()
    }

    /// The fixture set, before classification.
    pub fn fixture_items() -> Vec<SoftwareItem> {
        let mut items = Vec::new();

        // --- Safe: a classic Win32 program with a silent uninstaller --------
        let mut onedrive = SoftwareItem::new(
            "reg:hkcu:OneDriveSetup.exe",
            "Microsoft OneDrive",
            SourceKind::RegistryUninstall,
        );
        onedrive.version = Some("24.201.1006.0005".into());
        onedrive.publisher = Some("Microsoft Corporation".into());
        onedrive.scope = InstallScope::User;
        onedrive.arch = Architecture::X64;
        onedrive.size_bytes = Some(1_932_735_283);
        onedrive.install_date = Some("2026-03-14".into());
        onedrive.registry_key = Some(
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\OneDriveSetup.exe".into(),
        );
        onedrive.uninstall_string =
            Some(r"C:\Windows\SysWOW64\OneDriveSetup.exe /uninstall".into());
        onedrive.quiet_uninstall_string =
            Some(r"C:\Windows\SysWOW64\OneDriveSetup.exe /uninstall /silent".into());
        onedrive.install_location = Some(r"C:\Users\demo\AppData\Local\Microsoft\OneDrive".into());
        onedrive.executables = vec!["OneDrive.exe".into()];
        onedrive.extra.insert(
            "leftoverPaths".into(),
            join_list([
                r"C:\Users\demo\AppData\Local\Microsoft\OneDrive",
                r"C:\ProgramData\Microsoft OneDrive",
            ]),
        );
        onedrive.extra.insert(
            "leftoverRegistry".into(),
            join_list([r"HKCU\Software\Microsoft\OneDrive"]),
        );
        items.push(onedrive);

        // --- Safe: an AppX package ------------------------------------------
        let mut xbox = SoftwareItem::new(
            "appx:Microsoft.XboxApp",
            "Xbox Console Companion",
            SourceKind::AppxPackage,
        );
        xbox.version = Some("48.94.13001.0".into());
        xbox.publisher = Some("Microsoft Corporation".into());
        xbox.scope = InstallScope::AllUsers;
        xbox.arch = Architecture::X64;
        xbox.size_bytes = Some(184_549_376);
        xbox.package_full_name = Some("Microsoft.XboxApp_48.94.13001.0_x64__8wekyb3d8bbwe".into());
        xbox.package_family_name = Some("Microsoft.XboxApp_8wekyb3d8bbwe".into());
        items.push(xbox);

        let mut candy = SoftwareItem::new(
            "appx:king.com.CandyCrushSaga",
            "Candy Crush Saga",
            SourceKind::AppxPackage,
        );
        candy.publisher = Some("king.com".into());
        candy.scope = InstallScope::AllUsers;
        candy.size_bytes = Some(96_468_992);
        candy.package_full_name =
            Some("king.com.CandyCrushSaga_1.2420.1.0_x86__kgqvnymyfvs32".into());
        candy.package_family_name = Some("king.com.CandyCrushSaga_kgqvnymyfvs32".into());
        items.push(candy);

        let mut phone = SoftwareItem::new(
            "appx:Microsoft.YourPhone",
            "Phone Link",
            SourceKind::AppxPackage,
        );
        phone.publisher = Some("Microsoft Corporation".into());
        phone.size_bytes = Some(212_336_640);
        phone.package_full_name =
            Some("Microsoft.YourPhone_1.24022.83.0_x64__8wekyb3d8bbwe".into());
        phone.package_family_name = Some("Microsoft.YourPhone_8wekyb3d8bbwe".into());
        items.push(phone);

        // --- Safe: provisioned package that returns for new users -----------
        let mut news = SoftwareItem::new(
            "appxprov:Microsoft.BingNews",
            "Microsoft News",
            SourceKind::AppxProvisioned,
        );
        news.publisher = Some("Microsoft Corporation".into());
        news.package_full_name =
            Some("Microsoft.BingNews_4.55.62231.0_neutral_~_8wekyb3d8bbwe".into());
        news.package_family_name = Some("Microsoft.BingNews_8wekyb3d8bbwe".into());
        news.extra.insert(
            "provisionedPackageName".into(),
            "Microsoft.BingNews_4.55.62231.0_neutral_~_8wekyb3d8bbwe".into(),
        );
        items.push(news);

        // --- Safe: a telemetry service --------------------------------------
        let mut diagtrack = SoftwareItem::new(
            "svc:DiagTrack",
            "Connected User Experiences and Telemetry",
            SourceKind::WindowsService,
        );
        diagtrack.system_name = Some("DiagTrack".into());
        diagtrack.state = ItemState::Running;
        diagtrack.executables = vec!["svchost.exe".into()];
        items.push(diagtrack);

        // --- Safe: a telemetry scheduled task --------------------------------
        let mut ceip = SoftwareItem::new(
            r"task:\Microsoft\Windows\Customer Experience Improvement Program\Consolidator",
            "Consolidator",
            SourceKind::ScheduledTask,
        );
        ceip.system_name =
            Some(r"\Microsoft\Windows\Customer Experience Improvement Program\Consolidator".into());
        ceip.state = ItemState::Enabled;
        items.push(ceip);

        // --- Safe: a startup entry -------------------------------------------
        let mut startup = SoftwareItem::new(
            "startup:hkcu:Run:Spotify",
            "Spotify",
            SourceKind::StartupEntry,
        );
        startup.state = ItemState::Enabled;
        startup.extra.insert(
            "startupLocation".into(),
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run".into(),
        );
        items.push(startup);

        // --- Caution: a browser the shell leans on ---------------------------
        let mut edge = SoftwareItem::new(
            "reg:hklm64:Microsoft Edge",
            "Microsoft Edge",
            SourceKind::RegistryUninstall,
        );
        edge.version = Some("129.0.2792.52".into());
        edge.publisher = Some("Microsoft Corporation".into());
        edge.scope = InstallScope::Machine;
        edge.size_bytes = Some(627_048_448);
        edge.registry_key = Some(
            r"HKLM\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft Edge"
                .into(),
        );
        edge.uninstall_string = Some(
            r#"C:\Program Files (x86)\Microsoft\Edge\Application\129.0.2792.52\Installer\setup.exe --uninstall --system-level"#.into(),
        );
        edge.install_location = Some(r"C:\Program Files (x86)\Microsoft\Edge\Application".into());
        edge.executables = vec!["msedge.exe".into()];
        items.push(edge);

        // --- Caution: the default photo viewer -------------------------------
        let mut photos = SoftwareItem::new(
            "appx:Microsoft.Windows.Photos",
            "Photos",
            SourceKind::AppxPackage,
        );
        photos.publisher = Some("Microsoft Corporation".into());
        photos.size_bytes = Some(318_767_104);
        photos.package_full_name =
            Some("Microsoft.Windows.Photos_2024.11050.14001.0_x64__8wekyb3d8bbwe".into());
        photos.package_family_name = Some("Microsoft.Windows.Photos_8wekyb3d8bbwe".into());
        items.push(photos);

        // --- Critical: the antivirus -----------------------------------------
        let mut defender = SoftwareItem::new(
            "svc:WinDefend",
            "Microsoft Defender Antivirus Service",
            SourceKind::WindowsService,
        );
        defender.system_name = Some("WinDefend".into());
        defender.state = ItemState::Running;
        items.push(defender);

        // --- Critical: a core service -----------------------------------------
        let mut rpc = SoftwareItem::new(
            "svc:RpcSs",
            "Remote Procedure Call (RPC)",
            SourceKind::WindowsService,
        );
        rpc.system_name = Some("RpcSs".into());
        rpc.state = ItemState::Running;
        items.push(rpc);

        // --- Critical: a shared runtime ----------------------------------------
        let mut vcredist = SoftwareItem::new(
            "reg:hklm64:{e46eca4f-393b-40df-9f49-076faf788d83}",
            "Microsoft Visual C++ 2015-2022 Redistributable (x64) - 14.38.33130",
            SourceKind::RegistryUninstall,
        );
        vcredist.version = Some("14.38.33130.0".into());
        vcredist.publisher = Some("Microsoft Corporation".into());
        vcredist.scope = InstallScope::Machine;
        vcredist.size_bytes = Some(23_068_672);
        vcredist.registry_key = Some(
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\{e46eca4f-393b-40df-9f49-076faf788d83}".into(),
        );
        vcredist.uninstall_string =
            Some(r#""C:\ProgramData\Package Cache\{...}\VC_redist.x64.exe" /uninstall"#.into());
        items.push(vcredist);

        // --- Unknown: third-party software with no rule -------------------------
        let mut lob = SoftwareItem::new(
            "reg:hklm64:AcmeLedger",
            "Acme Ledger Desktop",
            SourceKind::RegistryUninstall,
        );
        lob.version = Some("7.2.1".into());
        lob.publisher = Some("Acme Industrial Software Ltd".into());
        lob.scope = InstallScope::Machine;
        lob.size_bytes = Some(478_150_656);
        lob.install_date = Some("2025-11-02".into());
        lob.registry_key =
            Some(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\AcmeLedger".into());
        lob.uninstall_string = Some(r"C:\Program Files\Acme\Ledger\uninstall.exe".into());
        lob.install_location = Some(r"C:\Program Files\Acme\Ledger".into());
        lob.executables = vec!["AcmeLedger.exe".into()];
        items.push(lob);

        // --- Not removable: an entry with no uninstall path ----------------------
        let mut orphan = SoftwareItem::new(
            "reg:hklm64:OrphanedEntry",
            "Legacy Toolbar (orphaned entry)",
            SourceKind::RegistryUninstall,
        );
        orphan.publisher = Some("Unknown".into());
        orphan.can_uninstall = false;
        orphan.registry_key =
            Some(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\OrphanedEntry".into());
        items.push(orphan);

        items
    }
}

impl PlatformBackend for MockBackend {
    fn platform_info(&self) -> PlatformInfo {
        PlatformInfo {
            platform: "mock".into(),
            os_description: "Mock Windows 11 Pro 23H2 (build 22631.3007)".into(),
            os_build: Some("22631.3007".into()),
            arch: "x86_64".into(),
            elevated: self.elevated,
            system_restore_available: !self.restore_point_fails,
        }
    }

    fn scan(
        &self,
        options: &ScanOptions,
        db: &SafetyDatabase,
        sink: &dyn EventSink,
    ) -> Result<ScanReport> {
        let mut report = ScanReport::new(options.clone(), now_rfc3339(), db.version().to_string());
        report.elevated = self.elevated;

        let passes = options.enabled_pass_count().max(1);
        let mut pass_index = 0;

        for kind in [
            SourceKind::RegistryUninstall,
            SourceKind::AppxPackage,
            SourceKind::AppxProvisioned,
            SourceKind::WindowsService,
            SourceKind::ScheduledTask,
            SourceKind::StartupEntry,
        ] {
            if !options.wants(kind) {
                continue;
            }
            pass_index += 1;
            sink.emit(Event::ScanPassStarted {
                pass: kind.slug().to_string(),
                index: pass_index,
                total: passes,
            });

            let mut found = 0;
            for mut item in Self::fixture_items()
                .into_iter()
                .filter(|i| i.source == kind)
            {
                if !options.include_non_removable && !item.can_uninstall && !item.can_disable {
                    continue;
                }
                db.apply(&mut item);
                report.items.push(item);
                found += 1;
            }

            sink.emit(Event::ScanPassFinished {
                pass: kind.slug().to_string(),
                found,
            });
        }

        Ok(report)
    }

    fn create_restore_point(&self, description: &str) -> Result<RestorePointInfo> {
        if self.restore_point_fails {
            return Err(crate::error::Error::RestorePoint(
                "System Protection is disabled on the system drive (mock)".into(),
            ));
        }
        self.journal
            .lock()
            .expect("journal mutex")
            .restore_points
            .push(description.to_string());
        Ok(RestorePointInfo {
            sequence_number: 42,
            description: description.to_string(),
            created_at: now_rfc3339(),
        })
    }

    fn backup_registry_keys(&self, keys: &[String], out_dir: &Path) -> Result<Vec<RegistryBackup>> {
        let mut journal = self.journal.lock().expect("journal mutex");
        Ok(keys
            .iter()
            .map(|key| {
                journal.backed_up_keys.push(key.clone());
                RegistryBackup {
                    key: key.clone(),
                    file: out_dir.join(format!(
                        "{}.reg",
                        key.replace(['\\', ':', ' ', '{', '}'], "_")
                    )),
                    bytes: 2_048,
                }
            })
            .collect())
    }

    fn kill_processes(&self, executables: &[String]) -> Result<Vec<KilledProcess>> {
        let mut journal = self.journal.lock().expect("journal mutex");
        let mut killed = Vec::new();
        for (i, exe) in executables.iter().enumerate() {
            // `svchost.exe` is shared: the real backend refuses to kill it, and
            // so does the mock, so tests see that behaviour.
            if exe.eq_ignore_ascii_case("svchost.exe") {
                continue;
            }
            journal.killed.push(exe.clone());
            killed.push(KilledProcess {
                pid: 4_000 + i as u32,
                name: exe.clone(),
                path: Some(format!(r"C:\Program Files\Mock\{exe}")),
            });
        }
        Ok(killed)
    }

    fn stop_services(&self, services: &[String]) -> Result<StepResult> {
        let mut journal = self.journal.lock().expect("journal mutex");
        for s in services {
            journal.services_changed.push((s.clone(), false));
        }
        Ok(StepResult::ok(format!(
            "stopped {} service(s)",
            services.len()
        )))
    }

    fn set_service_startup(&self, service: &str, enabled: bool) -> Result<StepResult> {
        self.journal
            .lock()
            .expect("journal mutex")
            .services_changed
            .push((service.to_string(), enabled));
        Ok(StepResult::ok(format!(
            "`{service}` start type set to {}",
            if enabled { "automatic" } else { "disabled" }
        )))
    }

    fn set_task_enabled(&self, task: &str, enabled: bool) -> Result<StepResult> {
        self.journal
            .lock()
            .expect("journal mutex")
            .tasks_changed
            .push((task.to_string(), enabled));
        Ok(StepResult::ok(format!(
            "task `{task}` {}",
            if enabled { "enabled" } else { "disabled" }
        )))
    }

    fn run_uninstaller(&self, command: &str, _silent: bool) -> Result<ExecOutcome> {
        self.journal
            .lock()
            .expect("journal mutex")
            .uninstallers_run
            .push(command.to_string());
        let fails = self
            .failing_uninstallers
            .iter()
            .any(|needle| command.contains(needle.as_str()));
        Ok(ExecOutcome {
            command: command.to_string(),
            exit_code: Some(if fails { 1603 } else { 0 }),
            stdout_tail: String::new(),
            stderr_tail: if fails {
                "fatal error during installation (mock)".into()
            } else {
                String::new()
            },
            timed_out: false,
            duration_ms: 120,
        })
    }

    fn remove_appx_package(&self, package_full_name: &str, all_users: bool) -> Result<StepResult> {
        self.journal
            .lock()
            .expect("journal mutex")
            .appx_removed
            .push(package_full_name.to_string());
        Ok(StepResult::ok(format!(
            "removed `{package_full_name}`{}",
            if all_users { " for all users" } else { "" }
        )))
    }

    fn remove_appx_provisioned(&self, package_name: &str) -> Result<StepResult> {
        self.journal
            .lock()
            .expect("journal mutex")
            .appx_removed
            .push(format!("provisioned:{package_name}"));
        Ok(StepResult::ok(format!("deprovisioned `{package_name}`")))
    }

    fn remove_startup_entry(&self, location: &str, name: &str) -> Result<StepResult> {
        Ok(StepResult::ok(format!(
            "removed startup entry `{name}` from {location}"
        )))
    }

    fn delete_paths(&self, paths: &[String], dry_run: bool) -> Result<CleanSummary> {
        let mut summary = CleanSummary::default();
        let mut journal = self.journal.lock().expect("journal mutex");
        for raw in paths {
            let expanded = self.expand_path(raw);
            match guard::validate_delete_path(&expanded) {
                Ok(()) => {
                    if !dry_run {
                        journal.paths_deleted.push(expanded.clone());
                    }
                    summary.removed.push(expanded);
                    summary.bytes_freed += 12_582_912;
                }
                Err(e) => summary.failed.push(format!("{expanded}: {e}")),
            }
        }
        Ok(summary)
    }

    fn delete_registry(
        &self,
        keys: &[String],
        values: &[String],
        dry_run: bool,
    ) -> Result<CleanSummary> {
        let mut summary = CleanSummary::default();
        let mut journal = self.journal.lock().expect("journal mutex");
        for key in keys {
            match guard::validate_delete_key(key) {
                Ok(()) => {
                    if !dry_run {
                        journal.keys_deleted.push(key.clone());
                    }
                    summary.removed.push(key.clone());
                }
                Err(e) => summary.failed.push(format!("{key}: {e}")),
            }
        }
        for value in values {
            match guard::validate_delete_value(value) {
                Ok(_) => summary.removed.push(value.clone()),
                Err(e) => summary.failed.push(format!("{value}: {e}")),
            }
        }
        Ok(summary)
    }

    fn expand_path(&self, raw: &str) -> String {
        raw.replace("%LOCALAPPDATA%", r"C:\Users\demo\AppData\Local")
            .replace("%APPDATA%", r"C:\Users\demo\AppData\Roaming")
            .replace("%PROGRAMDATA%", r"C:\ProgramData")
            .replace("%USERPROFILE%", r"C:\Users\demo")
            .replace("%PROGRAMFILES%", r"C:\Program Files")
            .replace("%SYSTEMROOT%", r"C:\Windows")
    }
}

/// Collects events for assertions.
#[derive(Debug, Default)]
pub struct RecordingSink {
    events: Mutex<Vec<Event>>,
}

impl RecordingSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<Event> {
        self.events.lock().expect("events mutex").clone()
    }

    /// Slugs of every step that reported a terminal status.
    pub fn finished_steps(&self) -> Vec<(String, StepStatus)> {
        self.events()
            .into_iter()
            .filter_map(|e| match e {
                Event::StepFinished { step, status, .. } => Some((step, status)),
                _ => None,
            })
            .collect()
    }
}

impl EventSink for RecordingSink {
    fn emit(&self, event: Event) {
        self.events.lock().expect("events mutex").push(event);
    }
}
