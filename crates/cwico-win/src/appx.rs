//! UWP / MSIX package enumeration and removal, via the WinRT packaging API.
//!
//! This is the modern half of "installed software" and where most Windows
//! 10/11 bloatware lives. Two distinctions matter:
//!
//! * An **installed package** belongs to one user. Removing it for one user
//!   leaves it installed for the others.
//! * A **provisioned package** is staged on the image and re-installed for
//!   every *new* user account. Removing the installed copy without
//!   deprovisioning is why bloatware "comes back" - the tool does both.
//!
//! Framework packages (`Microsoft.VCLibs`, `Microsoft.UI.Xaml`) are reported
//! but flagged, because other applications link against them at runtime.

use crate::naming::{architecture_label, family_from_full_name};
use cwico_core::backend::StepResult;
use cwico_core::{Error, Result};
use windows::core::HSTRING;
use windows::Management::Deployment::{PackageManager, RemovalOptions};

/// One AppX/MSIX package.
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// `Microsoft.YourPhone_1.24022.83.0_x64__8wekyb3d8bbwe`
    pub full_name: String,
    /// `Microsoft.YourPhone_8wekyb3d8bbwe`
    pub family_name: String,
    /// `Microsoft.YourPhone`
    pub name: String,
    /// Friendly name, when the manifest provides one.
    pub display_name: Option<String>,
    pub publisher: Option<String>,
    pub publisher_display_name: Option<String>,
    pub version: String,
    pub architecture: String,
    pub install_location: Option<String>,
    /// `true` for shared runtime packages other apps depend on.
    pub is_framework: bool,
    /// `true` when the package is part of the OS image and cannot be removed.
    pub is_system: bool,
    /// `true` when this package is also provisioned for new users.
    pub is_provisioned: bool,
}

fn manager() -> Result<PackageManager> {
    PackageManager::new().map_err(|e| Error::Appx {
        package: "<package manager>".into(),
        source_msg: format!("PackageManager::new failed: {e}"),
    })
}

/// Enumerate packages installed for the current user, or for every user when
/// `all_users` is set (which needs elevation).
pub fn enumerate(all_users: bool) -> Result<Vec<PackageInfo>> {
    let pm = manager()?;

    let provisioned: Vec<String> = list_provisioned_family_names(&pm).unwrap_or_default();

    let packages = if all_users {
        pm.FindPackages()
    } else {
        pm.FindPackagesByUserSecurityId(&HSTRING::new())
    }
    .map_err(|e| Error::Appx {
        package: "<enumerate>".into(),
        source_msg: format!("FindPackages failed: {e}"),
    })?;

    let mut out = Vec::new();
    for package in packages {
        let Ok(id) = package.Id() else { continue };

        let full_name = id.FullName().map(|s| s.to_string()).unwrap_or_default();
        if full_name.is_empty() {
            continue;
        }
        let family_name = id
            .FamilyName()
            .map(|s| s.to_string())
            .unwrap_or_else(|_| family_from_full_name(&full_name));
        let name = id.Name().map(|s| s.to_string()).unwrap_or_default();

        let version = id
            .Version()
            .map(|v| format!("{}.{}.{}.{}", v.Major, v.Minor, v.Build, v.Revision))
            .unwrap_or_default();

        let architecture = id
            .Architecture()
            .map(|a| architecture_label(a.0).to_string())
            .unwrap_or_else(|_| "unknown".into());

        // `DisplayName` is often empty for framework and system packages.
        let display_name = package
            .DisplayName()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        let publisher_display_name = package
            .PublisherDisplayName()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());

        // `InstalledPath` returns the path directly; `InstalledLocation`
        // would drag in the whole Storage_Search surface for the same string.
        let install_location = package
            .InstalledPath()
            .ok()
            .map(|p| p.to_string())
            .filter(|s| !s.is_empty());

        let is_framework = package.IsFramework().unwrap_or(false);

        // `SignatureKind == System` marks the packages Windows itself ships
        // and refuses to remove.
        let is_system = package
            .SignatureKind()
            .map(|kind| kind.0 == 0)
            .unwrap_or(false);

        out.push(PackageInfo {
            is_provisioned: provisioned
                .iter()
                .any(|f| f.eq_ignore_ascii_case(&family_name)),
            publisher: id.Publisher().ok().map(|s| s.to_string()),
            full_name,
            family_name,
            name,
            display_name,
            publisher_display_name,
            version,
            architecture,
            install_location,
            is_framework,
            is_system,
        });
    }

    out.sort_by_key(|p| p.name.to_lowercase());
    Ok(out)
}

fn list_provisioned_family_names(pm: &PackageManager) -> Result<Vec<String>> {
    let provisioned = pm.FindProvisionedPackages().map_err(|e| Error::Appx {
        package: "<provisioned>".into(),
        source_msg: format!("FindProvisionedPackages failed: {e}"),
    })?;

    let mut out = Vec::new();
    for package in provisioned {
        if let Ok(id) = package.Id() {
            if let Ok(family) = id.FamilyName() {
                out.push(family.to_string());
            }
        }
    }
    Ok(out)
}

/// Packages staged on the image for future users.
pub fn enumerate_provisioned() -> Result<Vec<PackageInfo>> {
    let pm = manager()?;
    let provisioned = pm.FindProvisionedPackages().map_err(|e| Error::Appx {
        package: "<provisioned>".into(),
        source_msg: format!("FindProvisionedPackages failed: {e}"),
    })?;

    let mut out = Vec::new();
    for package in provisioned {
        let Ok(id) = package.Id() else { continue };
        let full_name = id.FullName().map(|s| s.to_string()).unwrap_or_default();
        if full_name.is_empty() {
            continue;
        }
        out.push(PackageInfo {
            family_name: id
                .FamilyName()
                .map(|s| s.to_string())
                .unwrap_or_else(|_| family_from_full_name(&full_name)),
            name: id.Name().map(|s| s.to_string()).unwrap_or_default(),
            display_name: package
                .DisplayName()
                .ok()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty()),
            publisher: id.Publisher().ok().map(|s| s.to_string()),
            publisher_display_name: None,
            version: id
                .Version()
                .map(|v| format!("{}.{}.{}.{}", v.Major, v.Minor, v.Build, v.Revision))
                .unwrap_or_default(),
            architecture: id
                .Architecture()
                .map(|a| architecture_label(a.0).to_string())
                .unwrap_or_else(|_| "unknown".into()),
            install_location: None,
            is_framework: package.IsFramework().unwrap_or(false),
            is_system: false,
            is_provisioned: true,
            full_name,
        });
    }
    Ok(out)
}

/// Remove an installed package.
///
/// `all_users` maps to `RemovalOptions::RemoveForAllUsers`, which needs
/// elevation; without it the package is only removed for the caller.
pub fn remove_package(package_full_name: &str, all_users: bool) -> Result<StepResult> {
    let pm = manager()?;
    let name = HSTRING::from(package_full_name);

    let options = if all_users {
        RemovalOptions::RemoveForAllUsers
    } else {
        RemovalOptions::None
    };

    let operation = pm
        .RemovePackageWithOptionsAsync(&name, options)
        .map_err(|e| Error::Appx {
            package: package_full_name.to_string(),
            source_msg: format!("RemovePackageWithOptionsAsync failed to start: {e}"),
        })?;

    // `join` blocks until the deployment operation finishes. The engine
    // already runs each step sequentially, so blocking here is what we want.
    let result = operation.join().map_err(|e| Error::Appx {
        package: package_full_name.to_string(),
        source_msg: format!("removal did not complete: {e}"),
    })?;

    if let Ok(code) = result.ExtendedErrorCode() {
        if code.is_err() {
            let text = result
                .ErrorText()
                .map(|t| t.to_string())
                .unwrap_or_default();
            // 0x80073CF1 = the package is not installed. Idempotent success.
            if code.0 as u32 == 0x8007_3CF1 {
                return Ok(StepResult::skipped(format!(
                    "`{package_full_name}` was not installed"
                )));
            }
            return Err(Error::Appx {
                package: package_full_name.to_string(),
                source_msg: format!("removal failed (0x{:08X}): {text}", code.0),
            });
        }
    }

    Ok(StepResult::ok(format!(
        "removed `{package_full_name}`{}",
        if all_users { " for all users" } else { "" }
    )))
}

/// Deprovision a package so it is not installed for new user accounts.
///
/// Accepts either a family name or a full name.
pub fn deprovision(package_name: &str) -> Result<StepResult> {
    let pm = manager()?;
    let family = family_from_full_name(package_name);
    let name = HSTRING::from(family.as_str());

    let operation = pm
        .DeprovisionPackageForAllUsersAsync(&name)
        .map_err(|e| Error::Appx {
            package: family.clone(),
            source_msg: format!("DeprovisionPackageForAllUsersAsync failed to start: {e}"),
        })?;

    let result = operation.join().map_err(|e| Error::Appx {
        package: family.clone(),
        source_msg: format!("deprovisioning did not complete: {e}"),
    })?;

    if let Ok(code) = result.ExtendedErrorCode() {
        if code.is_err() {
            // Not provisioned in the first place: nothing to do.
            if code.0 as u32 == 0x8007_3CF1 {
                return Ok(StepResult::skipped(format!(
                    "`{family}` was not provisioned"
                )));
            }
            let text = result
                .ErrorText()
                .map(|t| t.to_string())
                .unwrap_or_default();
            return Err(Error::Appx {
                package: family,
                source_msg: format!("deprovisioning failed (0x{:08X}): {text}", code.0),
            });
        }
    }

    Ok(StepResult::ok(format!("deprovisioned `{family}`")))
}
