//! The scanner: turning six unrelated Windows subsystems into one list.
//!
//! Each pass produces [`cwico_core::SoftwareItem`]s, classifies them against
//! the safety database, and reports progress. A pass that fails records a
//! warning on the report and the scan continues — one unreadable registry
//! hive must not cost the user the other four hundred results.

use crate::naming::normalise_install_date;
use crate::registry::{RegKey, RegView};
use crate::{appx, process, services, startup, tasks};
use cwico_core::backend::{Event, EventSink};
use cwico_core::engine::now_rfc3339;
use cwico_core::model::{Architecture, InstallScope, ItemState, SoftwareItem, SourceKind};
use cwico_core::plan::join_list;
use cwico_core::safety::SafetyDatabase;
use cwico_core::scan::{ScanOptions, ScanReport};
use cwico_core::Result;
use std::path::PathBuf;
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};

/// Run every enabled pass.
pub fn scan(
    options: &ScanOptions,
    db: &SafetyDatabase,
    sink: &dyn EventSink,
    elevated: bool,
) -> Result<ScanReport> {
    let mut report = ScanReport::new(options.clone(), now_rfc3339(), db.version().to_string());
    report.elevated = elevated;

    if !elevated {
        report.warn(
            "elevation",
            "running without Administrator rights: machine-wide programs, service \
             configuration and provisioned packages are incomplete or read-only",
        );
    }

    let total = options.enabled_pass_count().max(1);
    let mut index = 0usize;

    let pass = |name: &str,
                index: &mut usize,
                report: &mut ScanReport,
                f: &mut dyn FnMut(&mut ScanReport) -> Result<usize>| {
        *index += 1;
        sink.emit(Event::ScanPassStarted {
            pass: name.to_string(),
            index: *index,
            total,
        });
        let found = match f(report) {
            Ok(n) => n,
            Err(e) => {
                report.warn(name, e.to_string());
                0
            }
        };
        sink.emit(Event::ScanPassFinished {
            pass: name.to_string(),
            found,
        });
    };

    if options.registry_programs {
        pass("registry", &mut index, &mut report, &mut |r| {
            scan_registry_programs(r, options, db)
        });
    }
    if options.appx_packages {
        pass("appx", &mut index, &mut report, &mut |r| {
            scan_appx(r, options, db, elevated)
        });
    }
    if options.appx_provisioned {
        pass("appx_provisioned", &mut index, &mut report, &mut |r| {
            scan_appx_provisioned(r, db)
        });
    }
    if options.services {
        pass("services", &mut index, &mut report, &mut |r| {
            scan_services(r, db)
        });
    }
    if options.scheduled_tasks {
        pass("scheduled_tasks", &mut index, &mut report, &mut |r| {
            scan_tasks(r, db)
        });
    }
    if options.startup_entries {
        pass("startup", &mut index, &mut report, &mut |r| {
            scan_startup(r, db)
        });
    }

    // Mark items whose processes are running, so the UI can warn that a
    // removal will close a window the user has open.
    annotate_running_processes(&mut report);

    Ok(report)
}

// ---------------------------------------------------------------------------
// Pass: registry-installed programs
// ---------------------------------------------------------------------------

fn scan_registry_programs(
    report: &mut ScanReport,
    options: &ScanOptions,
    db: &SafetyDatabase,
) -> Result<usize> {
    let mut found = 0;

    let roots: &[(
        &str,
        windows::Win32::System::Registry::HKEY,
        RegView,
        InstallScope,
    )] = &[
        (
            "hklm64",
            HKEY_LOCAL_MACHINE,
            RegView::Bits64,
            InstallScope::Machine,
        ),
        (
            "hklm32",
            HKEY_LOCAL_MACHINE,
            RegView::Bits32,
            InstallScope::Machine,
        ),
        (
            "hkcu",
            HKEY_CURRENT_USER,
            RegView::Bits64,
            InstallScope::User,
        ),
    ];

    const UNINSTALL_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";

    for (label, hive, view, scope) in roots {
        let Ok(root) = RegKey::open(*hive, UNINSTALL_PATH, *view, KEY_READ) else {
            report.warn(
                "registry",
                format!("could not open the {label} uninstall key"),
            );
            continue;
        };

        for subkey_name in root.subkey_names() {
            let full_subkey = format!("{UNINSTALL_PATH}\\{subkey_name}");
            let Ok(key) = RegKey::open(*hive, &full_subkey, *view, KEY_READ) else {
                continue;
            };

            let Some(display_name) = key.string("DisplayName") else {
                // No DisplayName means Add/Remove Programs hides it too:
                // usually an update or a component, not a product.
                continue;
            };

            let is_system_component = key.u32("SystemComponent").unwrap_or(0) == 1;
            if is_system_component && !options.include_system_components {
                continue;
            }
            // `ParentKeyName`/`ParentDisplayName` mark update entries that
            // belong to a product already listed.
            if key.string("ParentKeyName").is_some() && !options.include_system_components {
                continue;
            }
            if key.u32("ReleaseType").is_some() && key.string("ParentDisplayName").is_some() {
                continue;
            }

            let uninstall_string = key.string("UninstallString");
            let quiet = key.string("QuietUninstallString");
            let can_uninstall = uninstall_string.is_some() || quiet.is_some();
            if !can_uninstall && !options.include_non_removable {
                continue;
            }

            let hive_prefix = if *hive == HKEY_LOCAL_MACHINE {
                "HKLM"
            } else {
                "HKCU"
            };
            let registry_key = format!("{hive_prefix}\\{full_subkey}");

            let mut item = SoftwareItem::new(
                format!("reg:{label}:{subkey_name}"),
                display_name,
                SourceKind::RegistryUninstall,
            );
            item.version = key.string("DisplayVersion");
            item.publisher = key.string("Publisher");
            item.scope = *scope;
            item.arch = match view {
                RegView::Bits64 => Architecture::X64,
                RegView::Bits32 => Architecture::X86,
            };
            item.install_location = key.string("InstallLocation").map(PathBuf::from);
            item.uninstall_string = uninstall_string;
            item.quiet_uninstall_string = quiet;
            item.registry_key = Some(registry_key);
            item.can_uninstall = can_uninstall;
            item.install_date = key
                .string("InstallDate")
                .and_then(|d| normalise_install_date(&d));

            // `EstimatedSize` is in KiB.
            item.size_bytes = key
                .u32("EstimatedSize")
                .map(|kib| u64::from(kib) * 1024)
                .or_else(|| {
                    if options.measure_disk_usage {
                        item.install_location.as_ref().map(|p| directory_size(p))
                    } else {
                        None
                    }
                });

            if is_system_component {
                item.extra.insert("systemComponent".into(), "1".into());
            }
            if let Some(icon) = key.string("DisplayIcon") {
                item.extra.insert("displayIcon".into(), icon);
            }
            if let Some(location) = &item.install_location {
                item.extra.insert(
                    "leftoverPaths".into(),
                    join_list([location.to_string_lossy().into_owned()]),
                );
            }
            // The main executable, inferred from DisplayIcon or the install
            // location, so the kill step has something to match against.
            if let Some(exe) = infer_executable(&key, item.install_location.as_deref()) {
                item.executables.push(exe);
            }

            db.apply(&mut item);
            report.items.push(item);
            found += 1;
        }
    }

    Ok(found)
}

fn infer_executable(key: &RegKey, install_location: Option<&std::path::Path>) -> Option<String> {
    // `DisplayIcon` is usually `C:\Path\app.exe,0`.
    if let Some(icon) = key.string("DisplayIcon") {
        let path = icon
            .split(',')
            .next()
            .unwrap_or(&icon)
            .trim()
            .trim_matches('"');
        if path.to_ascii_lowercase().ends_with(".exe") {
            if let Some(name) = std::path::Path::new(path).file_name() {
                return Some(name.to_string_lossy().into_owned());
            }
        }
    }
    // Otherwise the largest .exe directly inside the install location is a
    // reasonable guess at the main program.
    let dir = install_location?;
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("exe"))
        })
        .max_by_key(|e| e.metadata().map(|m| m.len()).unwrap_or(0))
        .map(|e| e.file_name().to_string_lossy().into_owned())
}

fn directory_size(path: &std::path::Path) -> u64 {
    walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

// ---------------------------------------------------------------------------
// Pass: AppX / MSIX
// ---------------------------------------------------------------------------

fn scan_appx(
    report: &mut ScanReport,
    options: &ScanOptions,
    db: &SafetyDatabase,
    elevated: bool,
) -> Result<usize> {
    // Enumerating for all users needs elevation; fall back rather than fail.
    let packages = appx::enumerate(elevated)?;
    let mut found = 0;

    for package in packages {
        // Framework packages are dependencies, not products. They are shown
        // only in a deep scan, and the safety database classifies them
        // Critical so they can never be selected by accident.
        if package.is_framework && !options.include_system_components {
            continue;
        }

        let display = package
            .display_name
            .clone()
            .filter(|s| !s.starts_with("ms-resource:"))
            .unwrap_or_else(|| package.name.clone());

        let mut item = SoftwareItem::new(
            format!("appx:{}", package.family_name),
            display,
            SourceKind::AppxPackage,
        );
        item.version = Some(package.version.clone());
        item.publisher = package
            .publisher_display_name
            .clone()
            .or_else(|| package.publisher.clone());
        item.scope = if elevated {
            InstallScope::AllUsers
        } else {
            InstallScope::User
        };
        item.arch = match package.architecture.as_str() {
            "x64" => Architecture::X64,
            "x86" => Architecture::X86,
            "arm64" => Architecture::Arm64,
            "neutral" => Architecture::Neutral,
            _ => Architecture::Unknown,
        };
        item.package_full_name = Some(package.full_name.clone());
        item.package_family_name = Some(package.family_name.clone());
        item.install_location = package.install_location.as_deref().map(PathBuf::from);
        item.can_uninstall = !package.is_system;
        item.extra.insert(
            "leftoverPaths".into(),
            join_list([format!(r"%LOCALAPPDATA%\Packages\{}", package.family_name)]),
        );
        if package.is_framework {
            item.extra.insert("framework".into(), "1".into());
        }
        if package.is_system {
            item.extra.insert("systemPackage".into(), "1".into());
        }
        if package.is_provisioned {
            item.extra.insert("provisioned".into(), "1".into());
            item.extra
                .insert("provisionedPackageName".into(), package.family_name.clone());
        }
        if options.measure_disk_usage {
            if let Some(location) = &item.install_location {
                item.size_bytes = Some(directory_size(location));
            }
        }

        db.apply(&mut item);
        report.items.push(item);
        found += 1;
    }

    Ok(found)
}

fn scan_appx_provisioned(report: &mut ScanReport, db: &SafetyDatabase) -> Result<usize> {
    let packages = appx::enumerate_provisioned()?;
    let mut found = 0;

    // Only report a provisioned package that is not already listed as an
    // installed one, or the list shows every bloatware app twice.
    let already: Vec<String> = report
        .items
        .iter()
        .filter_map(|i| i.package_family_name.clone())
        .collect();

    for package in packages {
        if already
            .iter()
            .any(|f| f.eq_ignore_ascii_case(&package.family_name))
        {
            continue;
        }

        let mut item = SoftwareItem::new(
            format!("appxprov:{}", package.family_name),
            package
                .display_name
                .clone()
                .unwrap_or_else(|| package.name.clone()),
            SourceKind::AppxProvisioned,
        );
        item.version = Some(package.version.clone());
        item.publisher = package.publisher.clone();
        item.scope = InstallScope::AllUsers;
        item.package_full_name = Some(package.full_name.clone());
        item.package_family_name = Some(package.family_name.clone());
        item.extra
            .insert("provisionedPackageName".into(), package.family_name.clone());
        item.description = Some(cwico_core::model::LocalizedText::new(
            "Staged on this Windows image: it will be installed automatically for every \
             new user account until it is deprovisioned.",
            "Được nạp sẵn trong bản Windows này: nó sẽ tự động cài cho mọi tài khoản người \
             dùng mới cho đến khi bị gỡ khỏi image.",
        ));

        db.apply(&mut item);
        report.items.push(item);
        found += 1;
    }

    Ok(found)
}

// ---------------------------------------------------------------------------
// Pass: services
// ---------------------------------------------------------------------------

fn scan_services(report: &mut ScanReport, db: &SafetyDatabase) -> Result<usize> {
    let list = services::enumerate(false)?;
    let mut found = 0;

    for service in list {
        let mut item = SoftwareItem::new(
            format!("svc:{}", service.name),
            if service.display_name.is_empty() {
                service.name.clone()
            } else {
                service.display_name.clone()
            },
            SourceKind::WindowsService,
        );
        item.system_name = Some(service.name.clone());
        item.state = if service.running {
            ItemState::Running
        } else if service.start_type == services::StartType::Disabled {
            ItemState::Disabled
        } else {
            ItemState::Stopped
        };
        item.can_disable = true;
        // Services are disabled, never deleted; "uninstall" on a service means
        // stop-and-disable, which is the same reversible operation.
        item.can_uninstall = true;
        item.extra
            .insert("startType".into(), service.start_type.label().into());
        if let Some(path) = &service.binary_path {
            item.extra.insert("imagePath".into(), path.clone());
            if let Some(exe) = crate::cmdline::tokenize(path).first() {
                if let Some(name) = std::path::Path::new(exe).file_name() {
                    item.executables.push(name.to_string_lossy().into_owned());
                }
            }
        }
        item.registry_key = Some(format!(
            r"HKLM\SYSTEM\CurrentControlSet\Services\{}",
            service.name
        ));

        db.apply(&mut item);
        report.items.push(item);
        found += 1;
    }

    Ok(found)
}

// ---------------------------------------------------------------------------
// Pass: scheduled tasks
// ---------------------------------------------------------------------------

fn scan_tasks(report: &mut ScanReport, db: &SafetyDatabase) -> Result<usize> {
    let list = tasks::enumerate(false)?;
    let mut found = 0;

    for task in list {
        let mut item = SoftwareItem::new(
            format!("task:{}", task.path),
            if task.name.is_empty() {
                task.path.clone()
            } else {
                task.name.clone()
            },
            SourceKind::ScheduledTask,
        );
        item.system_name = Some(task.path.clone());
        item.state = if task.running {
            ItemState::Running
        } else if task.enabled {
            ItemState::Enabled
        } else {
            ItemState::Disabled
        };
        item.can_disable = true;
        item.can_uninstall = true;
        item.extra.insert("taskPath".into(), task.path.clone());

        db.apply(&mut item);
        report.items.push(item);
        found += 1;
    }

    Ok(found)
}

// ---------------------------------------------------------------------------
// Pass: startup entries
// ---------------------------------------------------------------------------

fn scan_startup(report: &mut ScanReport, db: &SafetyDatabase) -> Result<usize> {
    let list = startup::enumerate()?;
    let mut found = 0;

    for entry in list {
        let mut item = SoftwareItem::new(
            format!("startup:{}:{}", entry.location, entry.name),
            entry.name.clone(),
            SourceKind::StartupEntry,
        );
        item.state = ItemState::Enabled;
        item.can_disable = true;
        item.can_uninstall = true;
        item.extra
            .insert("startupLocation".into(), entry.location.clone());
        item.extra.insert("command".into(), entry.command.clone());
        if entry.target_missing {
            item.extra.insert("targetMissing".into(), "1".into());
            item.description = Some(cwico_core::model::LocalizedText::new(
                "Points at a program that no longer exists — leftover from an uninstalled \
                 application. Safe to remove.",
                "Trỏ tới một chương trình không còn tồn tại — tàn dư của ứng dụng đã gỡ. \
                 An toàn để xóa.",
            ));
        }
        if let Some(exe) = crate::cmdline::tokenize(&entry.command).first() {
            if let Some(name) = std::path::Path::new(exe).file_name() {
                item.executables.push(name.to_string_lossy().into_owned());
            }
        }
        if entry.location.starts_with("HK") {
            item.registry_key = Some(entry.location.clone());
        }

        db.apply(&mut item);
        report.items.push(item);
        found += 1;
    }

    Ok(found)
}

// ---------------------------------------------------------------------------
// Cross-cutting annotation
// ---------------------------------------------------------------------------

/// Flag items whose executables are running right now.
fn annotate_running_processes(report: &mut ScanReport) {
    let Ok(running) = process::enumerate() else {
        return;
    };
    let names: Vec<String> = running
        .iter()
        .map(|p| p.name.to_ascii_lowercase())
        .collect();

    for item in &mut report.items {
        if item.executables.is_empty() {
            continue;
        }
        let is_running = item
            .executables
            .iter()
            .any(|e| names.contains(&e.to_ascii_lowercase()));
        if is_running {
            item.extra.insert("running".into(), "1".into());
            if item.state == ItemState::Installed {
                item.state = ItemState::Running;
            }
        }
    }
}
