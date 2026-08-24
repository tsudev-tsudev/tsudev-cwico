//! Core data model: what a "removable thing" is, and how safe it is to touch.
//!
//! Everything the scanner finds - a classic Win32 program in the registry, a
//! UWP/AppX package, a Windows service, a scheduled task, an autostart entry -
//! is normalised into a single [`SoftwareItem`] so the UI, the safety
//! classifier and the uninstall engine only ever deal with one shape.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// A short bilingual string. The UI picks a field by the active locale; the
/// engine logs the English one so support bundles stay readable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LocalizedText {
    pub en: String,
    pub vi: String,
}

impl LocalizedText {
    pub fn new(en: impl Into<String>, vi: impl Into<String>) -> Self {
        Self {
            en: en.into(),
            vi: vi.into(),
        }
    }

    pub fn get(&self, locale: Locale) -> &str {
        match locale {
            Locale::En => &self.en,
            Locale::Vi => &self.vi,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    Vi,
    En,
}

// ---------------------------------------------------------------------------
// Where an item came from
// ---------------------------------------------------------------------------

/// The subsystem an item was discovered in. This drives which uninstall
/// strategy the engine picks, so it is deliberately fine-grained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// `HKLM/HKCU\...\Uninstall\*` - classic MSI / NSIS / Inno installers.
    RegistryUninstall,
    /// A UWP/MSIX package installed for one or more users.
    AppxPackage,
    /// A provisioned AppX package - reinstalled for every *new* user account
    /// unless it is deprovisioned too.
    AppxProvisioned,
    /// A Win32 service under `HKLM\SYSTEM\CurrentControlSet\Services`.
    WindowsService,
    /// An entry in the Task Scheduler tree.
    ScheduledTask,
    /// A `Run`/`RunOnce` registry value or a Startup-folder shortcut.
    StartupEntry,
    /// An on-demand Windows capability (`DISM /Get-Capabilities`).
    WindowsCapability,
    /// A Windows optional feature (`Get-WindowsOptionalFeature`).
    OptionalFeature,
    /// A residual folder/registry tree left behind by an already-gone product.
    Leftover,
}

impl SourceKind {
    /// Stable slug used in item ids and in the safety database.
    pub fn slug(self) -> &'static str {
        match self {
            SourceKind::RegistryUninstall => "reg",
            SourceKind::AppxPackage => "appx",
            SourceKind::AppxProvisioned => "appxprov",
            SourceKind::WindowsService => "svc",
            SourceKind::ScheduledTask => "task",
            SourceKind::StartupEntry => "startup",
            SourceKind::WindowsCapability => "cap",
            SourceKind::OptionalFeature => "feat",
            SourceKind::Leftover => "leftover",
        }
    }

    /// Whether this kind can be *disabled* (reversibly) as well as removed.
    /// Services, tasks and autostart entries can; an MSI program cannot.
    pub fn supports_disable(self) -> bool {
        matches!(
            self,
            SourceKind::WindowsService | SourceKind::ScheduledTask | SourceKind::StartupEntry
        )
    }
}

/// Machine-wide vs per-user installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallScope {
    Machine,
    User,
    /// AppX packages provisioned for every user on the image.
    AllUsers,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    X86,
    X64,
    Arm64,
    Neutral,
    Unknown,
}

/// Current runtime state, for the kinds where "running" is meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemState {
    Running,
    Stopped,
    Enabled,
    Disabled,
    Installed,
    Unknown,
}

// ---------------------------------------------------------------------------
// Safety classification
// ---------------------------------------------------------------------------

/// How dangerous it is to remove an item. This is the single most important
/// value in the whole tool: it is what stands between a user clicking
/// "select all" and an unbootable machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SafetyClass {
    /// Removing this leaves Windows fully functional. OneDrive, Xbox, Candy
    /// Crush, Bing News, Skype. Bulk-selectable.
    Safe,
    /// Removal works but costs a secondary feature the user may want:
    /// Windows Camera, Media Player, Edge, Cortana. Requires an explicit
    /// per-item confirmation in the UI.
    Caution,
    /// Not classified. Third-party software the database has never seen.
    /// Treated as `Caution` for gating purposes but shown distinctly, because
    /// "unknown" and "risky" are different facts.
    #[default]
    Unknown,
    /// Removal breaks Windows, security, or the ability to log in.
    /// Defender, File Explorer, Settings, CoreShell, RPC, WinLogon.
    /// **Hard-blocked** by the engine - the UI cannot override this.
    Critical,
}

impl SafetyClass {
    pub fn slug(self) -> &'static str {
        match self {
            SafetyClass::Safe => "safe",
            SafetyClass::Caution => "caution",
            SafetyClass::Unknown => "unknown",
            SafetyClass::Critical => "critical",
        }
    }

    /// `true` when the engine refuses the removal no matter what the caller
    /// asks for. Only `Critical` is unconditionally blocked.
    pub fn is_blocked(self) -> bool {
        matches!(self, SafetyClass::Critical)
    }

    /// `true` when the UI must collect a deliberate, per-item confirmation
    /// before the item may be queued.
    pub fn needs_confirmation(self) -> bool {
        matches!(self, SafetyClass::Caution | SafetyClass::Unknown)
    }

    /// `true` when the item may be included by a "select all safe" action.
    pub fn is_bulk_selectable(self) -> bool {
        matches!(self, SafetyClass::Safe)
    }
}

// ---------------------------------------------------------------------------
// The item itself
// ---------------------------------------------------------------------------

/// One removable / disableable thing found on the system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareItem {
    /// Stable synthetic id, e.g. `appx:Microsoft.YourPhone` or
    /// `reg:hklm64:{90160000-008C-0000-1000-0000000FF1CE}`. Stable across
    /// scans so UI selections survive a rescan.
    pub id: String,

    /// `DisplayName` for registry entries, package display name for AppX,
    /// display name for services.
    pub name: String,

    /// `DisplayVersion`.
    pub version: Option<String>,

    /// `Publisher`.
    pub publisher: Option<String>,

    pub source: SourceKind,
    pub scope: InstallScope,
    pub arch: Architecture,
    pub state: ItemState,

    /// Verdict from the safety database, resolved at scan time.
    pub safety: SafetyClass,

    /// Why it carries that verdict - shown in the UI next to the badge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_reason: Option<LocalizedText>,

    /// Human-facing description of what the software actually does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,

    /// `InstallLocation`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_location: Option<PathBuf>,

    /// `UninstallString` - the vendor's own uninstall command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uninstall_string: Option<String>,

    /// `QuietUninstallString` - preferred when present, it needs no UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiet_uninstall_string: Option<String>,

    /// `EstimatedSize` (registry, in KiB) normalised to bytes, or the measured
    /// size of the install location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,

    /// `InstallDate`, normalised to `YYYY-MM-DD` when parseable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_date: Option<String>,

    /// The registry key this item was read from, used for the `.reg` backup.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_key: Option<String>,

    /// AppX `PackageFullName`, needed by `Remove-AppxPackage`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_full_name: Option<String>,

    /// AppX `PackageFamilyName`, used to locate per-user state folders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_family_name: Option<String>,

    /// Service short name / scheduled task full path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_name: Option<String>,

    /// Executables belonging to this item, matched against running processes
    /// before uninstalling.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub executables: Vec<String>,

    /// Base64 PNG of the item's icon, filled in lazily by the UI layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_png_base64: Option<String>,

    /// `true` when the item can be reversibly turned off instead of removed.
    pub can_disable: bool,

    /// `true` when the engine has a working removal path for this item.
    /// A registry entry with no `UninstallString` and no install location is
    /// discoverable but not removable.
    pub can_uninstall: bool,

    /// Free-form extras (`SystemComponent`, `WindowsInstaller`, task triggers…)
    /// kept for the details pane and the audit log.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, String>,
}

impl SoftwareItem {
    /// Minimal constructor; callers fill the optional fields.
    pub fn new(id: impl Into<String>, name: impl Into<String>, source: SourceKind) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: None,
            publisher: None,
            source,
            scope: InstallScope::Unknown,
            arch: Architecture::Unknown,
            state: ItemState::Installed,
            safety: SafetyClass::Unknown,
            safety_reason: None,
            description: None,
            install_location: None,
            uninstall_string: None,
            quiet_uninstall_string: None,
            size_bytes: None,
            install_date: None,
            registry_key: None,
            package_full_name: None,
            package_family_name: None,
            system_name: None,
            executables: Vec::new(),
            icon_png_base64: None,
            can_disable: source.supports_disable(),
            can_uninstall: true,
            extra: BTreeMap::new(),
        }
    }

    /// The command the engine should run for the vendor uninstall step,
    /// preferring the silent variant.
    pub fn preferred_uninstall_command(&self) -> Option<&str> {
        self.quiet_uninstall_string
            .as_deref()
            .or(self.uninstall_string.as_deref())
    }

    /// `true` when a silent uninstall is possible without synthesising flags.
    pub fn has_native_silent_uninstall(&self) -> bool {
        self.quiet_uninstall_string.is_some()
    }

    /// Lowercased haystack used by the UI's search box.
    pub fn search_haystack(&self) -> String {
        let mut s = self.name.to_lowercase();
        for opt in [
            self.publisher.as_deref(),
            self.version.as_deref(),
            self.system_name.as_deref(),
            self.package_family_name.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            s.push(' ');
            s.push_str(&opt.to_lowercase());
        }
        s
    }
}

// ---------------------------------------------------------------------------
// Actions
// ---------------------------------------------------------------------------

/// What the user asked to do with an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Reversibly turn it off: stop + disable a service, disable a task,
    /// remove an autostart entry (backed up first).
    Disable,
    /// Undo a previous `Disable`.
    Enable,
    /// Run the full uninstall flow, leaving leftovers alone.
    Uninstall,
    /// Uninstall, then sweep folders and registry residue.
    UninstallAndDeepClean,
    /// Sweep residue only - for items already uninstalled elsewhere.
    DeepCleanOnly,
}

impl Action {
    /// `true` when this action mutates the system irreversibly enough to
    /// warrant a restore point.
    pub fn is_destructive(self) -> bool {
        matches!(
            self,
            Action::Uninstall | Action::UninstallAndDeepClean | Action::DeepCleanOnly
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_is_blocked_and_never_bulk_selectable() {
        assert!(SafetyClass::Critical.is_blocked());
        assert!(!SafetyClass::Critical.is_bulk_selectable());
        assert!(!SafetyClass::Safe.is_blocked());
        assert!(SafetyClass::Safe.is_bulk_selectable());
    }

    #[test]
    fn unknown_needs_confirmation_but_is_not_blocked() {
        assert!(SafetyClass::Unknown.needs_confirmation());
        assert!(!SafetyClass::Unknown.is_blocked());
        assert!(!SafetyClass::Unknown.is_bulk_selectable());
    }

    #[test]
    fn safety_class_orders_safe_below_critical() {
        assert!(SafetyClass::Safe < SafetyClass::Caution);
        assert!(SafetyClass::Caution < SafetyClass::Critical);
    }

    #[test]
    fn quiet_uninstall_string_wins() {
        let mut item = SoftwareItem::new("reg:x", "X", SourceKind::RegistryUninstall);
        item.uninstall_string = Some("setup.exe /uninstall".into());
        assert_eq!(
            item.preferred_uninstall_command(),
            Some("setup.exe /uninstall")
        );
        item.quiet_uninstall_string = Some("setup.exe /uninstall /S".into());
        assert_eq!(
            item.preferred_uninstall_command(),
            Some("setup.exe /uninstall /S")
        );
        assert!(item.has_native_silent_uninstall());
    }

    #[test]
    fn only_stateful_kinds_support_disable() {
        assert!(SourceKind::WindowsService.supports_disable());
        assert!(SourceKind::ScheduledTask.supports_disable());
        assert!(!SourceKind::RegistryUninstall.supports_disable());
        assert!(!SourceKind::AppxPackage.supports_disable());
    }
}
