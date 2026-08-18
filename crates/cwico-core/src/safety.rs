//! The safety database: the rule set that decides whether an item is
//! `Safe`, `Caution` or `Critical`.
//!
//! Design rules that must not be relaxed:
//!
//! 1. **Fail safe, not permissive.** An item that matches nothing is
//!    [`SafetyClass::Unknown`], never `Safe`.
//! 2. **Critical always wins.** If an item matches both a `Safe` rule and a
//!    `Critical` rule, the verdict is `Critical`. Severity beats specificity.
//! 3. **The database is always present.** A vetted copy is compiled into the
//!    binary with `include_str!`, so a missing or corrupt `data/safety-db.json`
//!    downgrades to the built-in set rather than leaving the tool unguarded.

use crate::error::{Error, Result};
use crate::model::{LocalizedText, SafetyClass, SoftwareItem, SourceKind};
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// The vetted rule set shipped with the binary. Loading an external file
/// replaces it; a broken external file falls back to it.
pub const BUILTIN_DB_JSON: &str = include_str!("../../../data/safety-db.json");

// ---------------------------------------------------------------------------
// Serialised shape
// ---------------------------------------------------------------------------

/// Residue a rule knows about: what the product leaves behind after its own
/// uninstaller has run. Consumed by the deep-clean engine.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leftovers {
    /// Filesystem paths, with `%ENVVAR%` and `{USERPROFILE}` style expansion
    /// performed by the platform backend.
    #[serde(default)]
    pub paths: Vec<String>,
    /// Registry keys, e.g. `HKCU\Software\Microsoft\OneDrive`.
    #[serde(default)]
    pub registry: Vec<String>,
    /// Registry *values* to clear without deleting their parent key,
    /// written as `HKCU\Path\To\Key::ValueName`.
    #[serde(default)]
    pub registry_values: Vec<String>,
}

impl Leftovers {
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty() && self.registry.is_empty() && self.registry_values.is_empty()
    }
}

/// How a rule recognises an item.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchSpec {
    /// Restrict the rule to these source kinds. Empty means "any kind".
    #[serde(default)]
    pub kinds: Vec<SourceKind>,
    /// Case-insensitive exact match against any identity key of the item.
    #[serde(default)]
    pub exact: Vec<String>,
    /// Case-insensitive substring match against any identity key.
    #[serde(default)]
    pub contains: Vec<String>,
    /// Case-insensitive regex against any identity key.
    #[serde(default)]
    pub regex: Vec<String>,
    /// Additionally require the publisher to contain this string.
    #[serde(default)]
    pub publisher_contains: Option<String>,
}

/// One classification rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyRule {
    /// Stable rule id, e.g. `ms.onedrive`. Referenced by logs and by tests.
    pub id: String,
    pub class: SafetyClass,
    #[serde(rename = "match")]
    pub match_spec: MatchSpec,
    /// Friendly product name for the UI, when nicer than the raw DisplayName.
    #[serde(default)]
    pub label: Option<LocalizedText>,
    /// Why this class was assigned. Always shown next to the badge.
    pub reason: LocalizedText,
    /// What the software is for.
    #[serde(default)]
    pub description: Option<LocalizedText>,
    /// Free-form tags (`bloatware`, `telemetry`, `gaming`, `oem`) for filters.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Processes to terminate before uninstalling.
    #[serde(default)]
    pub processes: Vec<String>,
    /// Services to stop and disable alongside the item.
    #[serde(default)]
    pub services: Vec<String>,
    /// Scheduled tasks to disable alongside the item.
    #[serde(default)]
    pub tasks: Vec<String>,
    /// Known residue for the deep-clean pass.
    #[serde(default)]
    pub leftovers: Leftovers,
}

/// The on-disk database document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyDbDocument {
    pub schema_version: u32,
    pub version: String,
    pub updated: String,
    #[serde(default)]
    pub source_notes: Vec<String>,
    pub rules: Vec<SafetyRule>,
}

// ---------------------------------------------------------------------------
// Compiled database
// ---------------------------------------------------------------------------

/// A rule that matched, with the score that decided which reason text wins.
#[derive(Debug, Clone)]
pub struct Verdict {
    pub class: SafetyClass,
    pub rule_id: Option<String>,
    pub label: Option<LocalizedText>,
    pub reason: Option<LocalizedText>,
    pub description: Option<LocalizedText>,
    pub tags: Vec<String>,
    pub processes: Vec<String>,
    pub services: Vec<String>,
    pub tasks: Vec<String>,
    pub leftovers: Leftovers,
}

impl Verdict {
    /// The verdict for an item no rule recognised.
    pub fn unknown() -> Self {
        Self {
            class: SafetyClass::Unknown,
            rule_id: None,
            label: None,
            reason: Some(LocalizedText::new(
                "Not in the safety database — third-party or uncommon software. \
                 Review before removing.",
                "Chưa có trong cơ sở dữ liệu an toàn — phần mềm bên thứ ba hoặc ít gặp. \
                 Hãy xem kỹ trước khi gỡ.",
            )),
            description: None,
            tags: Vec::new(),
            processes: Vec::new(),
            services: Vec::new(),
            tasks: Vec::new(),
            leftovers: Leftovers::default(),
        }
    }
}

/// Compiled, query-ready safety database.
pub struct SafetyDatabase {
    doc: SafetyDbDocument,
    /// `lowercased exact token -> rule indices`
    exact_index: HashMap<String, Vec<usize>>,
    /// `lowercased substring -> rule index`, scanned linearly (the set is small
    /// and the scan runs once per item).
    contains_index: Vec<(String, usize)>,
    /// One compiled `RegexSet` over every rule regex, plus the rule index each
    /// pattern belongs to.
    regex_set: RegexSet,
    regex_owner: Vec<usize>,
}

impl std::fmt::Debug for SafetyDatabase {
    /// Summarised rather than exhaustive: dumping several hundred compiled
    /// rules into a log line helps nobody.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (safe, caution, critical) = self.class_counts();
        f.debug_struct("SafetyDatabase")
            .field("version", &self.doc.version)
            .field("updated", &self.doc.updated)
            .field("rules", &self.doc.rules.len())
            .field("safe", &safe)
            .field("caution", &caution)
            .field("critical", &critical)
            .finish()
    }
}

impl SafetyDatabase {
    /// Compile the vetted rule set that ships inside the binary.
    ///
    /// # Panics
    /// Only if the crate was built with a malformed `data/safety-db.json`,
    /// which the `builtin_database_is_valid` test rules out at CI time.
    pub fn builtin() -> Self {
        Self::from_json(BUILTIN_DB_JSON)
            .expect("compiled-in safety database must parse; run `cargo test -p cwico-core`")
    }

    /// Load an external database, falling back to the built-in set if the file
    /// is missing or unparseable. The fallback is logged, never silent.
    pub fn load_or_builtin(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        match std::fs::read_to_string(path).map_err(|e| Error::io(path, e)) {
            Ok(text) => match Self::from_json(&text) {
                Ok(db) => {
                    tracing::info!(
                        path = %path.display(),
                        version = %db.doc.version,
                        rules = db.doc.rules.len(),
                        "loaded external safety database"
                    );
                    db
                }
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "safety database is malformed; using the built-in rule set"
                    );
                    Self::builtin()
                }
            },
            Err(e) => {
                tracing::debug!(
                    path = %path.display(),
                    error = %e,
                    "no external safety database; using the built-in rule set"
                );
                Self::builtin()
            }
        }
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let doc: SafetyDbDocument =
            serde_json::from_str(json).map_err(|e| Error::SafetyDatabase(e.to_string()))?;
        Self::compile(doc)
    }

    fn compile(doc: SafetyDbDocument) -> Result<Self> {
        let mut exact_index: HashMap<String, Vec<usize>> = HashMap::new();
        let mut contains_index: Vec<(String, usize)> = Vec::new();
        let mut patterns: Vec<String> = Vec::new();
        let mut regex_owner: Vec<usize> = Vec::new();

        let mut seen_ids = HashMap::new();
        for (idx, rule) in doc.rules.iter().enumerate() {
            if let Some(prev) = seen_ids.insert(rule.id.clone(), idx) {
                return Err(Error::SafetyDatabase(format!(
                    "duplicate rule id `{}` at indices {prev} and {idx}",
                    rule.id
                )));
            }
            if rule.match_spec.exact.is_empty()
                && rule.match_spec.contains.is_empty()
                && rule.match_spec.regex.is_empty()
            {
                return Err(Error::SafetyDatabase(format!(
                    "rule `{}` has no match criteria; it would never fire",
                    rule.id
                )));
            }
            for token in &rule.match_spec.exact {
                exact_index
                    .entry(token.to_lowercase())
                    .or_default()
                    .push(idx);
            }
            for token in &rule.match_spec.contains {
                contains_index.push((token.to_lowercase(), idx));
            }
            for pattern in &rule.match_spec.regex {
                patterns.push(format!("(?i){pattern}"));
                regex_owner.push(idx);
            }
        }

        let regex_set = RegexSet::new(&patterns)
            .map_err(|e| Error::SafetyDatabase(format!("invalid regex in safety database: {e}")))?;

        Ok(Self {
            doc,
            exact_index,
            contains_index,
            regex_set,
            regex_owner,
        })
    }

    pub fn version(&self) -> &str {
        &self.doc.version
    }

    pub fn updated(&self) -> &str {
        &self.doc.updated
    }

    pub fn rules(&self) -> &[SafetyRule] {
        &self.doc.rules
    }

    pub fn rule(&self, id: &str) -> Option<&SafetyRule> {
        self.doc.rules.iter().find(|r| r.id == id)
    }

    /// Count of rules per class — surfaced in the UI's "about" panel so users
    /// can see how much protection is actually loaded.
    pub fn class_counts(&self) -> (usize, usize, usize) {
        let mut safe = 0;
        let mut caution = 0;
        let mut critical = 0;
        for rule in &self.doc.rules {
            match rule.class {
                SafetyClass::Safe => safe += 1,
                SafetyClass::Caution => caution += 1,
                SafetyClass::Critical => critical += 1,
                SafetyClass::Unknown => {}
            }
        }
        (safe, caution, critical)
    }

    /// The identity strings a rule may match against, lowercased.
    fn identity_keys(item: &SoftwareItem) -> Vec<String> {
        let mut keys = Vec::with_capacity(6);
        keys.push(item.name.to_lowercase());
        for opt in [
            item.system_name.as_deref(),
            item.package_family_name.as_deref(),
            item.package_full_name.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            keys.push(opt.to_lowercase());
        }
        // The AppX family name without its publisher hash: `Microsoft.YourPhone`
        // out of `Microsoft.YourPhone_8wekyb3d8bbwe`.
        if let Some(family) = item.package_family_name.as_deref() {
            if let Some((stem, _hash)) = family.rsplit_once('_') {
                keys.push(stem.to_lowercase());
            }
        }
        if let Some(exe) = item.executables.first() {
            keys.push(exe.to_lowercase());
        }
        keys.dedup();
        keys
    }

    fn rule_applies_to_kind(rule: &SafetyRule, source: SourceKind) -> bool {
        rule.match_spec.kinds.is_empty() || rule.match_spec.kinds.contains(&source)
    }

    fn publisher_ok(rule: &SafetyRule, item: &SoftwareItem) -> bool {
        match rule.match_spec.publisher_contains.as_deref() {
            None => true,
            Some(needle) => item
                .publisher
                .as_deref()
                .is_some_and(|p| p.to_lowercase().contains(&needle.to_lowercase())),
        }
    }

    /// Classify an item.
    ///
    /// Precedence is severity first (`Critical` > `Caution` > `Safe`), then
    /// specificity within the winning class (exact match > regex > substring,
    /// longer pattern wins ties) so the most precise reason text is shown.
    pub fn classify(&self, item: &SoftwareItem) -> Verdict {
        let keys = Self::identity_keys(item);
        // (rule index, specificity score)
        let mut hits: Vec<(usize, u32)> = Vec::new();

        for key in &keys {
            if let Some(indices) = self.exact_index.get(key) {
                for &idx in indices {
                    hits.push((idx, 1_000_000 + key.len() as u32));
                }
            }
            for (needle, idx) in &self.contains_index {
                if key.contains(needle.as_str()) {
                    hits.push((*idx, needle.len() as u32));
                }
            }
            for pattern_idx in self.regex_set.matches(key).into_iter() {
                let idx = self.regex_owner[pattern_idx];
                hits.push((idx, 500_000));
            }
        }

        // Drop rules that do not apply to this kind or publisher.
        hits.retain(|(idx, _)| {
            let rule = &self.doc.rules[*idx];
            Self::rule_applies_to_kind(rule, item.source) && Self::publisher_ok(rule, item)
        });

        if hits.is_empty() {
            return Verdict::unknown();
        }

        // Severity first: Critical beats everything, then Caution, then Safe.
        let severity = |c: SafetyClass| match c {
            SafetyClass::Critical => 3,
            SafetyClass::Caution => 2,
            SafetyClass::Safe => 1,
            SafetyClass::Unknown => 0,
        };
        let top_severity = hits
            .iter()
            .map(|(idx, _)| severity(self.doc.rules[*idx].class))
            .max()
            .unwrap_or(0);

        let best = hits
            .iter()
            .filter(|(idx, _)| severity(self.doc.rules[*idx].class) == top_severity)
            .max_by_key(|(_, score)| *score)
            .map(|(idx, _)| *idx)
            .expect("hits is non-empty and top_severity came from it");

        // Merge the remediation hints of *every* matching rule of the winning
        // class: a product often has one broad rule and one specific rule, and
        // both know about residue worth cleaning.
        let mut verdict = {
            let rule = &self.doc.rules[best];
            Verdict {
                class: rule.class,
                rule_id: Some(rule.id.clone()),
                label: rule.label.clone(),
                reason: Some(rule.reason.clone()),
                description: rule.description.clone(),
                tags: rule.tags.clone(),
                processes: rule.processes.clone(),
                services: rule.services.clone(),
                tasks: rule.tasks.clone(),
                leftovers: rule.leftovers.clone(),
            }
        };

        let mut merged: Vec<usize> = hits
            .iter()
            .filter(|(idx, _)| severity(self.doc.rules[*idx].class) == top_severity)
            .map(|(idx, _)| *idx)
            .collect();
        merged.sort_unstable();
        merged.dedup();
        for idx in merged {
            if idx == best {
                continue;
            }
            let rule = &self.doc.rules[idx];
            verdict.tags.extend(rule.tags.iter().cloned());
            verdict.processes.extend(rule.processes.iter().cloned());
            verdict.services.extend(rule.services.iter().cloned());
            verdict.tasks.extend(rule.tasks.iter().cloned());
            verdict
                .leftovers
                .paths
                .extend(rule.leftovers.paths.iter().cloned());
            verdict
                .leftovers
                .registry
                .extend(rule.leftovers.registry.iter().cloned());
            verdict
                .leftovers
                .registry_values
                .extend(rule.leftovers.registry_values.iter().cloned());
        }
        dedup_sorted(&mut verdict.tags);
        dedup_sorted(&mut verdict.processes);
        dedup_sorted(&mut verdict.services);
        dedup_sorted(&mut verdict.tasks);
        dedup_sorted(&mut verdict.leftovers.paths);
        dedup_sorted(&mut verdict.leftovers.registry);
        dedup_sorted(&mut verdict.leftovers.registry_values);

        verdict
    }

    /// Classify in place, filling `safety`, `safety_reason` and `description`.
    pub fn apply(&self, item: &mut SoftwareItem) -> Verdict {
        let verdict = self.classify(item);
        item.safety = verdict.class;
        item.safety_reason = verdict.reason.clone();
        if item.description.is_none() {
            item.description = verdict.description.clone();
        }
        if let Some(label) = &verdict.label {
            // The rule's label names the *group* ("Bing apps", "OEM antivirus
            // trials"), which is useful for grouping the list — but the name
            // shown to the user stays the system's own DisplayName. Renaming
            // "Candy Crush Saga" to "Preinstalled third-party games" would
            // hide the very thing the user is looking for.
            item.extra.insert("groupLabel".into(), label.en.clone());
            item.extra.insert("groupLabelVi".into(), label.vi.clone());
        }
        if let Some(rule_id) = &verdict.rule_id {
            item.extra.insert("safetyRuleId".into(), rule_id.clone());
        }
        for exe in &verdict.processes {
            if !item.executables.iter().any(|e| e.eq_ignore_ascii_case(exe)) {
                item.executables.push(exe.clone());
            }
        }

        // Hand the planner everything the rule knows: which services and tasks
        // travel with this product, and what residue to sweep. `plan.rs` reads
        // these back out of `extra`.
        for (field, values) in [
            ("relatedServices", &verdict.services),
            ("relatedTasks", &verdict.tasks),
            ("leftoverPaths", &verdict.leftovers.paths),
            ("leftoverRegistry", &verdict.leftovers.registry),
            ("leftoverRegistryValues", &verdict.leftovers.registry_values),
        ] {
            if values.is_empty() {
                continue;
            }
            // Merge with anything the scanner already discovered rather than
            // replacing it: the registry knows the real InstallLocation, the
            // rule knows the residue the uninstaller forgets.
            let merged = match item.extra.get(field) {
                Some(existing) => {
                    let mut all: Vec<String> = existing
                        .lines()
                        .map(str::to_string)
                        .chain(values.iter().cloned())
                        .filter(|l| !l.trim().is_empty())
                        .collect();
                    all.sort_by_key(|s| s.to_lowercase());
                    all.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
                    all.join("\n")
                }
                None => values.join("\n"),
            };
            item.extra.insert(field.to_string(), merged);
        }

        verdict
    }
}

fn dedup_sorted(v: &mut Vec<String>) {
    v.sort_by_key(|s| s.to_lowercase());
    v.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::SourceKind;

    fn item(name: &str, source: SourceKind) -> SoftwareItem {
        SoftwareItem::new(format!("{}:{name}", source.slug()), name, source)
    }

    #[test]
    fn builtin_database_is_valid() {
        let db = SafetyDatabase::from_json(BUILTIN_DB_JSON)
            .expect("data/safety-db.json must parse and compile");
        let (safe, caution, critical) = db.class_counts();
        assert!(safe > 20, "expected a substantial safe list, got {safe}");
        assert!(caution > 5, "expected a caution list, got {caution}");
        assert!(
            critical > 15,
            "the critical protection list is the whole point; got {critical}"
        );
    }

    #[test]
    fn every_rule_has_a_bilingual_reason() {
        let db = SafetyDatabase::builtin();
        for rule in db.rules() {
            assert!(
                !rule.reason.en.trim().is_empty(),
                "rule `{}` has no English reason",
                rule.id
            );
            assert!(
                !rule.reason.vi.trim().is_empty(),
                "rule `{}` has no Vietnamese reason",
                rule.id
            );
        }
    }

    #[test]
    fn onedrive_is_safe_to_remove() {
        let db = SafetyDatabase::builtin();
        let mut it = item("Microsoft OneDrive", SourceKind::RegistryUninstall);
        it.executables.push("OneDrive.exe".into());
        let v = db.classify(&it);
        assert_eq!(v.class, SafetyClass::Safe, "verdict: {v:?}");
        assert!(
            !v.leftovers.is_empty(),
            "OneDrive rule must know its residue"
        );
    }

    #[test]
    fn defender_and_explorer_are_critical() {
        let db = SafetyDatabase::builtin();
        for name in [
            "Windows Defender",
            "Microsoft Defender Antivirus",
            "Windows Explorer",
        ] {
            let it = item(name, SourceKind::RegistryUninstall);
            assert_eq!(
                db.classify(&it).class,
                SafetyClass::Critical,
                "`{name}` must be Critical"
            );
        }
    }

    #[test]
    fn core_services_are_critical() {
        let db = SafetyDatabase::builtin();
        for svc in ["RpcSs", "WinDefend", "Winmgmt", "LSM", "DcomLaunch"] {
            let mut it = item(svc, SourceKind::WindowsService);
            it.system_name = Some(svc.to_string());
            assert_eq!(
                db.classify(&it).class,
                SafetyClass::Critical,
                "service `{svc}` must be Critical"
            );
        }
    }

    #[test]
    fn unmatched_software_is_unknown_not_safe() {
        let db = SafetyDatabase::builtin();
        let it = item(
            "Totally Bespoke Line-Of-Business App",
            SourceKind::RegistryUninstall,
        );
        let v = db.classify(&it);
        assert_eq!(v.class, SafetyClass::Unknown);
        assert!(v.reason.is_some());
    }

    #[test]
    fn critical_beats_safe_when_both_match() {
        // A crafted database where one rule would clear the item and another
        // protects it. The protective verdict has to win.
        let json = r#"{
          "schemaVersion": 1, "version": "test", "updated": "2026-01-01",
          "rules": [
            {"id":"a","class":"safe","match":{"contains":["widget"]},
             "reason":{"en":"safe","vi":"an toàn"}},
            {"id":"b","class":"critical","match":{"exact":["core widget host"]},
             "reason":{"en":"critical","vi":"quan trọng"}}
          ]
        }"#;
        let db = SafetyDatabase::from_json(json).unwrap();
        let it = item("Core Widget Host", SourceKind::RegistryUninstall);
        let v = db.classify(&it);
        assert_eq!(v.class, SafetyClass::Critical);
        assert_eq!(v.rule_id.as_deref(), Some("b"));
    }

    #[test]
    fn kind_restriction_is_honoured() {
        let json = r#"{
          "schemaVersion": 1, "version": "test", "updated": "2026-01-01",
          "rules": [
            {"id":"svc-only","class":"critical","match":{"kinds":["windows_service"],"exact":["foo"]},
             "reason":{"en":"c","vi":"c"}}
          ]
        }"#;
        let db = SafetyDatabase::from_json(json).unwrap();

        let mut svc = item("foo", SourceKind::WindowsService);
        svc.system_name = Some("foo".into());
        assert_eq!(db.classify(&svc).class, SafetyClass::Critical);

        let reg = item("foo", SourceKind::RegistryUninstall);
        assert_eq!(db.classify(&reg).class, SafetyClass::Unknown);
    }

    #[test]
    fn appx_family_name_matches_without_publisher_hash() {
        let db = SafetyDatabase::builtin();
        let mut it = item("Your Phone", SourceKind::AppxPackage);
        it.package_family_name = Some("Microsoft.YourPhone_8wekyb3d8bbwe".into());
        assert_eq!(db.classify(&it).class, SafetyClass::Safe);
    }

    #[test]
    fn duplicate_rule_ids_are_rejected() {
        let json = r#"{
          "schemaVersion": 1, "version": "t", "updated": "2026-01-01",
          "rules": [
            {"id":"dup","class":"safe","match":{"exact":["a"]},"reason":{"en":"x","vi":"x"}},
            {"id":"dup","class":"safe","match":{"exact":["b"]},"reason":{"en":"x","vi":"x"}}
          ]
        }"#;
        assert!(SafetyDatabase::from_json(json).is_err());
    }

    #[test]
    fn rule_without_criteria_is_rejected() {
        let json = r#"{
          "schemaVersion": 1, "version": "t", "updated": "2026-01-01",
          "rules": [{"id":"empty","class":"safe","match":{},"reason":{"en":"x","vi":"x"}}]
        }"#;
        assert!(SafetyDatabase::from_json(json).is_err());
    }

    #[test]
    fn apply_populates_the_item() {
        let db = SafetyDatabase::builtin();
        let mut it = item("Microsoft OneDrive", SourceKind::RegistryUninstall);
        db.apply(&mut it);
        assert_eq!(it.safety, SafetyClass::Safe);
        assert!(it.safety_reason.is_some());
        assert!(it.extra.contains_key("safetyRuleId"));
        // The rule's residue and process hints reach the planner.
        assert!(it.extra.contains_key("leftoverPaths"));
        assert!(it.executables.iter().any(|e| e == "OneDrive.exe"));
    }

    #[test]
    fn a_group_rule_never_renames_the_item() {
        // `thirdparty.game.bloat` is labelled "Preinstalled third-party games
        // & trials". The row must still say "Candy Crush Saga", or the user
        // cannot find what they came to remove.
        let db = SafetyDatabase::builtin();
        let mut it = item("Candy Crush Saga", SourceKind::AppxPackage);
        it.package_family_name = Some("king.com.CandyCrushSaga_kgqvnymyfvs32".into());
        db.apply(&mut it);

        assert_eq!(it.name, "Candy Crush Saga");
        assert_eq!(it.safety, SafetyClass::Safe);
        assert!(it.extra["groupLabel"].contains("third-party"));
    }

    #[test]
    fn rule_hints_merge_with_scanner_findings_instead_of_replacing_them() {
        let db = SafetyDatabase::builtin();
        let mut it = item("Microsoft OneDrive", SourceKind::RegistryUninstall);
        it.extra
            .insert("leftoverPaths".into(), r"C:\Vendor\Discovered".into());
        db.apply(&mut it);

        let paths = &it.extra["leftoverPaths"];
        assert!(paths.contains("Discovered"), "scanner finding was dropped");
        assert!(paths.contains("OneDrive"), "rule finding was dropped");
    }
}
