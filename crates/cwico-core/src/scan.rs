//! Scan configuration and the report the UI renders.

use crate::model::{Locale, SafetyClass, SoftwareItem, SourceKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Which subsystems to sweep, and how deeply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanOptions {
    /// `HKLM`/`HKCU` uninstall keys, both 64-bit and 32-bit views.
    pub registry_programs: bool,
    /// UWP/MSIX packages installed for the current user.
    pub appx_packages: bool,
    /// Provisioned AppX packages that reappear for each new user account.
    pub appx_provisioned: bool,
    /// Windows services.
    pub services: bool,
    /// Task Scheduler entries.
    pub scheduled_tasks: bool,
    /// `Run`/`RunOnce` values and Startup-folder shortcuts.
    pub startup_entries: bool,
    /// Optional Windows features (`Hyper-V`, `WSL`, `.NET 3.5`…).
    pub optional_features: bool,
    /// Sweep for residue of software that is already gone. Slow: it walks
    /// Program Files, ProgramData and the per-user AppData trees.
    pub leftovers: bool,

    /// Include entries flagged `SystemComponent=1`, which Add/Remove Programs
    /// hides. They are usually runtimes and driver packages.
    pub include_system_components: bool,
    /// Include entries with no `UninstallString` (discoverable, not removable).
    pub include_non_removable: bool,
    /// Measure `InstallLocation` on disk when `EstimatedSize` is absent.
    /// Accurate but adds several seconds on large installs.
    pub measure_disk_usage: bool,
}

impl Default for ScanOptions {
    /// The default sweep covers everything a user would expect to see in a
    /// debloater, minus the two slow passes.
    fn default() -> Self {
        Self {
            registry_programs: true,
            appx_packages: true,
            appx_provisioned: true,
            services: true,
            scheduled_tasks: true,
            startup_entries: true,
            optional_features: true,
            leftovers: false,
            include_system_components: false,
            include_non_removable: false,
            measure_disk_usage: false,
        }
    }
}

impl ScanOptions {
    /// Everything, including the slow passes. What the "Deep scan" button does.
    pub fn deep() -> Self {
        Self {
            leftovers: true,
            include_system_components: true,
            include_non_removable: true,
            measure_disk_usage: true,
            ..Self::default()
        }
    }

    /// Installed programs only — the fastest useful scan.
    pub fn quick() -> Self {
        Self {
            registry_programs: true,
            appx_packages: true,
            appx_provisioned: false,
            services: false,
            scheduled_tasks: false,
            startup_entries: false,
            optional_features: false,
            leftovers: false,
            include_system_components: false,
            include_non_removable: false,
            measure_disk_usage: false,
        }
    }

    pub fn wants(&self, kind: SourceKind) -> bool {
        match kind {
            SourceKind::RegistryUninstall => self.registry_programs,
            SourceKind::AppxPackage => self.appx_packages,
            SourceKind::AppxProvisioned => self.appx_provisioned,
            SourceKind::WindowsService => self.services,
            SourceKind::ScheduledTask => self.scheduled_tasks,
            SourceKind::StartupEntry => self.startup_entries,
            SourceKind::OptionalFeature | SourceKind::WindowsCapability => self.optional_features,
            SourceKind::Leftover => self.leftovers,
        }
    }

    /// Number of passes enabled, used to weight progress reporting.
    pub fn enabled_pass_count(&self) -> usize {
        [
            self.registry_programs,
            self.appx_packages,
            self.appx_provisioned,
            self.services,
            self.scheduled_tasks,
            self.startup_entries,
            self.optional_features,
            self.leftovers,
        ]
        .iter()
        .filter(|b| **b)
        .count()
    }
}

/// A non-fatal problem during a scan: one registry hive that could not be
/// opened should not lose the other 400 results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanWarning {
    pub pass: String,
    pub message: String,
}

/// Counts for the UI's summary strip.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStats {
    pub total: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub by_safety: BTreeMap<String, usize>,
    /// Sum of `size_bytes` across items that reported one.
    pub reclaimable_bytes: u64,
    pub duration_ms: u64,
}

/// The result of one sweep.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    /// RFC 3339 timestamp.
    pub started_at: String,
    pub options: ScanOptions,
    pub items: Vec<SoftwareItem>,
    #[serde(default)]
    pub warnings: Vec<ScanWarning>,
    pub stats: ScanStats,
    /// Version of the safety database that classified these items.
    pub safety_db_version: String,
    /// `false` when the process was not elevated: the report is then partial
    /// (HKLM writes, service configuration and provisioned packages are
    /// invisible or unusable without elevation).
    pub elevated: bool,
}

impl ScanReport {
    pub fn new(options: ScanOptions, started_at: String, safety_db_version: String) -> Self {
        Self {
            started_at,
            options,
            items: Vec::new(),
            warnings: Vec::new(),
            stats: ScanStats::default(),
            safety_db_version,
            elevated: false,
        }
    }

    pub fn warn(&mut self, pass: impl Into<String>, message: impl Into<String>) {
        let w = ScanWarning {
            pass: pass.into(),
            message: message.into(),
        };
        tracing::warn!(pass = %w.pass, message = %w.message, "scan warning");
        self.warnings.push(w);
    }

    /// Recompute [`ScanStats`] from `items`. Call once the passes are done.
    pub fn finalize(&mut self, duration_ms: u64) {
        let mut stats = ScanStats {
            total: self.items.len(),
            duration_ms,
            ..Default::default()
        };
        for item in &self.items {
            *stats
                .by_kind
                .entry(item.source.slug().to_string())
                .or_insert(0) += 1;
            *stats
                .by_safety
                .entry(item.safety.slug().to_string())
                .or_insert(0) += 1;
            if let Some(bytes) = item.size_bytes {
                stats.reclaimable_bytes = stats.reclaimable_bytes.saturating_add(bytes);
            }
        }
        self.stats = stats;
    }

    pub fn get(&self, id: &str) -> Option<&SoftwareItem> {
        self.items.iter().find(|i| i.id == id)
    }

    /// Items a "select all safe" action may tick, sorted by reclaimable size.
    pub fn bulk_selectable(&self) -> Vec<&SoftwareItem> {
        let mut v: Vec<&SoftwareItem> = self
            .items
            .iter()
            .filter(|i| i.safety.is_bulk_selectable() && i.can_uninstall)
            .collect();
        v.sort_by_key(|i| std::cmp::Reverse(i.size_bytes.unwrap_or(0)));
        v
    }
}

/// UI-side filtering, kept in the core so the CLI and the GUI agree.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemFilter {
    /// Free-text query matched against name, publisher, version and identifiers.
    #[serde(default)]
    pub query: String,
    /// Empty means "all kinds".
    #[serde(default)]
    pub kinds: Vec<SourceKind>,
    /// Empty means "all classes".
    #[serde(default)]
    pub safety: Vec<SafetyClass>,
    /// Only items carrying every one of these tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Hide items the engine cannot act on.
    #[serde(default)]
    pub removable_only: bool,
}

impl ItemFilter {
    pub fn matches(&self, item: &SoftwareItem) -> bool {
        if !self.kinds.is_empty() && !self.kinds.contains(&item.source) {
            return false;
        }
        if !self.safety.is_empty() && !self.safety.contains(&item.safety) {
            return false;
        }
        if self.removable_only && !item.can_uninstall && !item.can_disable {
            return false;
        }
        if !self.query.trim().is_empty() {
            let haystack = item.search_haystack();
            // Every whitespace-separated term must appear: typing
            // "microsoft xbox" narrows rather than widens.
            if !self
                .query
                .to_lowercase()
                .split_whitespace()
                .all(|term| haystack.contains(term))
            {
                return false;
            }
        }
        true
    }

    pub fn apply<'a>(&self, items: &'a [SoftwareItem]) -> Vec<&'a SoftwareItem> {
        items.iter().filter(|i| self.matches(i)).collect()
    }
}

/// Sort orders offered by the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SortBy {
    #[default]
    Name,
    SizeDesc,
    Safety,
    Publisher,
    InstallDate,
    Kind,
}

pub fn sort_items(items: &mut [&SoftwareItem], by: SortBy, locale: Locale) {
    let _ = locale; // reserved for locale-aware collation
    match by {
        SortBy::Name => items.sort_by_key(|i| i.name.to_lowercase()),
        SortBy::SizeDesc => items.sort_by(|a, b| {
            b.size_bytes
                .unwrap_or(0)
                .cmp(&a.size_bytes.unwrap_or(0))
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }),
        // Safe first: that is the list a user acts on.
        SortBy::Safety => items.sort_by(|a, b| {
            a.safety
                .cmp(&b.safety)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }),
        SortBy::Publisher => items.sort_by_key(|i| {
            (
                i.publisher.clone().unwrap_or_default().to_lowercase(),
                i.name.to_lowercase(),
            )
        }),
        SortBy::InstallDate => items.sort_by(|a, b| {
            b.install_date
                .cmp(&a.install_date)
                .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }),
        SortBy::Kind => items.sort_by_key(|i| (i.source, i.name.to_lowercase())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceKind;

    fn mk(name: &str, safety: SafetyClass, kind: SourceKind) -> SoftwareItem {
        let mut i = SoftwareItem::new(format!("{}:{name}", kind.slug()), name, kind);
        i.safety = safety;
        i
    }

    #[test]
    fn quick_scan_skips_the_slow_passes() {
        let q = ScanOptions::quick();
        assert!(!q.leftovers);
        assert!(!q.measure_disk_usage);
        assert!(q.registry_programs);
    }

    #[test]
    fn deep_scan_enables_everything() {
        let d = ScanOptions::deep();
        assert!(d.leftovers && d.include_system_components && d.measure_disk_usage);
        assert_eq!(d.enabled_pass_count(), 8);
    }

    #[test]
    fn filter_requires_every_query_term() {
        let mut item = mk(
            "Microsoft Xbox App",
            SafetyClass::Safe,
            SourceKind::AppxPackage,
        );
        item.publisher = Some("Microsoft Corporation".into());

        let f = ItemFilter {
            query: "microsoft xbox".into(),
            ..Default::default()
        };
        assert!(f.matches(&item));

        let f = ItemFilter {
            query: "microsoft photoshop".into(),
            ..Default::default()
        };
        assert!(!f.matches(&item), "unrelated term must exclude the item");
    }

    #[test]
    fn filter_by_safety_and_kind() {
        let safe_appx = mk("A", SafetyClass::Safe, SourceKind::AppxPackage);
        let crit_svc = mk("B", SafetyClass::Critical, SourceKind::WindowsService);
        let items = vec![safe_appx, crit_svc];

        let f = ItemFilter {
            safety: vec![SafetyClass::Safe],
            ..Default::default()
        };
        assert_eq!(f.apply(&items).len(), 1);

        let f = ItemFilter {
            kinds: vec![SourceKind::WindowsService],
            ..Default::default()
        };
        assert_eq!(f.apply(&items)[0].name, "B");
    }

    #[test]
    fn bulk_selection_never_includes_critical_or_caution() {
        let mut report = ScanReport::new(ScanOptions::default(), "now".into(), "test".into());
        report.items = vec![
            mk("Safe", SafetyClass::Safe, SourceKind::AppxPackage),
            mk("Caution", SafetyClass::Caution, SourceKind::AppxPackage),
            mk(
                "Critical",
                SafetyClass::Critical,
                SourceKind::WindowsService,
            ),
            mk(
                "Unknown",
                SafetyClass::Unknown,
                SourceKind::RegistryUninstall,
            ),
        ];
        let picked: Vec<&str> = report
            .bulk_selectable()
            .iter()
            .map(|i| i.name.as_str())
            .collect();
        assert_eq!(picked, vec!["Safe"]);
    }

    #[test]
    fn finalize_computes_stats() {
        let mut report = ScanReport::new(ScanOptions::default(), "now".into(), "test".into());
        let mut a = mk("A", SafetyClass::Safe, SourceKind::AppxPackage);
        a.size_bytes = Some(1_000);
        let mut b = mk("B", SafetyClass::Safe, SourceKind::AppxPackage);
        b.size_bytes = Some(2_500);
        report.items = vec![
            a,
            b,
            mk("C", SafetyClass::Critical, SourceKind::WindowsService),
        ];
        report.finalize(42);

        assert_eq!(report.stats.total, 3);
        assert_eq!(report.stats.reclaimable_bytes, 3_500);
        assert_eq!(report.stats.by_kind["appx"], 2);
        assert_eq!(report.stats.by_safety["critical"], 1);
        assert_eq!(report.stats.duration_ms, 42);
    }

    #[test]
    fn sort_by_safety_puts_safe_first() {
        let items = [
            mk("Z", SafetyClass::Critical, SourceKind::WindowsService),
            mk("A", SafetyClass::Safe, SourceKind::AppxPackage),
        ];
        let mut refs: Vec<&SoftwareItem> = items.iter().collect();
        sort_items(&mut refs, SortBy::Safety, Locale::Vi);
        assert_eq!(refs[0].safety, SafetyClass::Safe);
    }
}
