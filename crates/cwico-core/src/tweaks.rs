//! System tweaks: the reversible half of the toolkit.
//!
//! This is where the original `Optimize_Win11_For_Dev.ps1` ended up. That
//! script applied twelve fixed groups of changes with no way to inspect,
//! choose or undo them. Here each change is an individually selectable
//! [`Tweak`] carrying its own safety class, an explicit revert path and a
//! bilingual explanation of what it costs.
//!
//! Tweaks are data, not code: they live in `data/tweaks.json` and are
//! compiled in with `include_str!`, so adding one is a data change.

use crate::error::{Error, Result};
use crate::model::{LocalizedText, SafetyClass};
use serde::{Deserialize, Serialize};

/// The vetted tweak catalogue shipped with the binary.
pub const BUILTIN_TWEAKS_JSON: &str = include_str!("../../../data/tweaks.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TweakCategory {
    Privacy,
    Performance,
    Explorer,
    Gaming,
    Network,
    Developer,
    Interface,
    Cleanup,
}

impl TweakCategory {
    pub fn slug(self) -> &'static str {
        match self {
            TweakCategory::Privacy => "privacy",
            TweakCategory::Performance => "performance",
            TweakCategory::Explorer => "explorer",
            TweakCategory::Gaming => "gaming",
            TweakCategory::Network => "network",
            TweakCategory::Developer => "developer",
            TweakCategory::Interface => "interface",
            TweakCategory::Cleanup => "cleanup",
        }
    }
}

/// Registry value types, mirroring `REG_*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegValueKind {
    Dword,
    Qword,
    String,
    ExpandString,
    MultiString,
    Binary,
}

/// One atomic change a tweak makes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TweakEffect {
    /// Write a registry value, creating the key if necessary.
    SetRegistryValue {
        key: String,
        name: String,
        kind: RegValueKind,
        /// Rendered as a string; the backend converts by `kind`.
        value: String,
    },
    /// Delete a registry value. Used by revert paths that restore "unset".
    DeleteRegistryValue { key: String, name: String },
    /// Set a service's start type.
    SetServiceStartup {
        service: String,
        /// `automatic`, `manual`, `disabled`.
        start_type: String,
    },
    /// Enable or disable a Windows optional feature.
    SetOptionalFeature { feature: String, enabled: bool },
    /// Activate a power scheme by GUID or well-known alias.
    SetPowerScheme { scheme: String },
    /// Run a short, well-known command. Restricted to an allow-list in the
    /// backend — this is not a general shell escape.
    RunCommand {
        program: String,
        #[serde(default)]
        args: Vec<String>,
    },
}

impl TweakEffect {
    pub fn slug(&self) -> &'static str {
        match self {
            TweakEffect::SetRegistryValue { .. } => "set_registry_value",
            TweakEffect::DeleteRegistryValue { .. } => "delete_registry_value",
            TweakEffect::SetServiceStartup { .. } => "set_service_startup",
            TweakEffect::SetOptionalFeature { .. } => "set_optional_feature",
            TweakEffect::SetPowerScheme { .. } => "set_power_scheme",
            TweakEffect::RunCommand { .. } => "run_command",
        }
    }

    /// The registry key this effect touches, so the run can back it up first.
    pub fn registry_key(&self) -> Option<&str> {
        match self {
            TweakEffect::SetRegistryValue { key, .. }
            | TweakEffect::DeleteRegistryValue { key, .. } => Some(key),
            _ => None,
        }
    }
}

/// One user-selectable system change.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tweak {
    /// Stable id, e.g. `privacy.telemetry.off`.
    pub id: String,
    pub category: TweakCategory,
    pub title: LocalizedText,
    pub description: LocalizedText,
    /// What it costs. `Safe` tweaks lose nothing; `Caution` tweaks trade a
    /// feature for the benefit; `Critical` is not used here — a tweak that
    /// dangerous does not belong in the catalogue at all.
    pub safety: SafetyClass,
    /// `true` when the change only takes effect after a restart or a shell
    /// restart.
    #[serde(default)]
    pub requires_restart: bool,
    /// Effects applied when the tweak is turned on.
    pub apply: Vec<TweakEffect>,
    /// Effects that restore the Windows default. An empty list means the
    /// tweak is one-way, which the UI must say out loud.
    #[serde(default)]
    pub revert: Vec<TweakEffect>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether this tweak is part of the "recommended" preset.
    #[serde(default)]
    pub recommended: bool,
}

impl Tweak {
    pub fn is_reversible(&self) -> bool {
        !self.revert.is_empty()
    }

    /// Every registry key the tweak touches in either direction.
    pub fn registry_keys(&self) -> Vec<String> {
        let mut keys: Vec<String> = self
            .apply
            .iter()
            .chain(self.revert.iter())
            .filter_map(|e| e.registry_key())
            .map(str::to_string)
            .collect();
        keys.sort();
        keys.dedup();
        keys
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TweakCatalog {
    pub schema_version: u32,
    pub version: String,
    pub tweaks: Vec<Tweak>,
}

impl TweakCatalog {
    pub fn builtin() -> Self {
        Self::from_json(BUILTIN_TWEAKS_JSON)
            .expect("compiled-in tweak catalogue must parse; run `cargo test -p cwico-core`")
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let catalog: TweakCatalog =
            serde_json::from_str(json).map_err(|e| Error::SafetyDatabase(e.to_string()))?;
        catalog.validate()?;
        Ok(catalog)
    }

    fn validate(&self) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for tweak in &self.tweaks {
            if !seen.insert(&tweak.id) {
                return Err(Error::SafetyDatabase(format!(
                    "duplicate tweak id `{}`",
                    tweak.id
                )));
            }
            if tweak.apply.is_empty() {
                return Err(Error::SafetyDatabase(format!(
                    "tweak `{}` has no effects",
                    tweak.id
                )));
            }
            if tweak.safety == SafetyClass::Critical {
                return Err(Error::SafetyDatabase(format!(
                    "tweak `{}` is classified Critical; such a change must not be offered",
                    tweak.id
                )));
            }
        }
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Tweak> {
        self.tweaks.iter().find(|t| t.id == id)
    }

    pub fn by_category(&self, category: TweakCategory) -> Vec<&Tweak> {
        self.tweaks
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// The preset the UI ticks when a user picks "Recommended".
    pub fn recommended(&self) -> Vec<&Tweak> {
        self.tweaks.iter().filter(|t| t.recommended).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_catalog_is_valid() {
        let catalog = TweakCatalog::builtin();
        assert!(
            catalog.tweaks.len() >= 20,
            "the catalogue should cover the original script; got {}",
            catalog.tweaks.len()
        );
    }

    #[test]
    fn every_tweak_is_bilingual_and_explained() {
        for tweak in TweakCatalog::builtin().tweaks {
            for (field, text) in [("title", &tweak.title), ("description", &tweak.description)] {
                assert!(
                    !text.en.trim().is_empty() && !text.vi.trim().is_empty(),
                    "tweak `{}` is missing a {field} translation",
                    tweak.id
                );
            }
        }
    }

    #[test]
    fn no_tweak_is_classified_critical() {
        for tweak in TweakCatalog::builtin().tweaks {
            assert_ne!(tweak.safety, SafetyClass::Critical, "tweak {}", tweak.id);
        }
    }

    #[test]
    fn recommended_preset_contains_only_safe_tweaks() {
        let catalog = TweakCatalog::builtin();
        let recommended = catalog.recommended();
        assert!(
            !recommended.is_empty(),
            "there must be a recommended preset"
        );
        for tweak in recommended {
            assert_eq!(
                tweak.safety,
                SafetyClass::Safe,
                "`{}` is recommended but not Safe",
                tweak.id
            );
        }
    }

    #[test]
    fn registry_tweaks_are_reversible() {
        // A registry write we cannot undo is a trap. Every tweak whose apply
        // path is purely registry writes must ship a revert path.
        for tweak in TweakCatalog::builtin().tweaks {
            let only_registry = tweak
                .apply
                .iter()
                .all(|e| matches!(e, TweakEffect::SetRegistryValue { .. }));
            if only_registry {
                assert!(
                    tweak.is_reversible(),
                    "registry-only tweak `{}` has no revert path",
                    tweak.id
                );
            }
        }
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let json = r#"{"schemaVersion":1,"version":"t","tweaks":[
          {"id":"a","category":"privacy","title":{"en":"a","vi":"a"},
           "description":{"en":"d","vi":"d"},"safety":"safe",
           "apply":[{"op":"setRegistryValue","key":"HKCU\\S","name":"n","kind":"dword","value":"1"}]},
          {"id":"a","category":"privacy","title":{"en":"a","vi":"a"},
           "description":{"en":"d","vi":"d"},"safety":"safe",
           "apply":[{"op":"setRegistryValue","key":"HKCU\\S","name":"n","kind":"dword","value":"1"}]}
        ]}"#;
        assert!(TweakCatalog::from_json(json).is_err());
    }

    #[test]
    fn categories_are_all_populated() {
        let catalog = TweakCatalog::builtin();
        for category in [
            TweakCategory::Privacy,
            TweakCategory::Performance,
            TweakCategory::Explorer,
            TweakCategory::Gaming,
            TweakCategory::Developer,
            TweakCategory::Interface,
        ] {
            assert!(
                !catalog.by_category(category).is_empty(),
                "category `{}` has no tweaks",
                category.slug()
            );
        }
    }
}
