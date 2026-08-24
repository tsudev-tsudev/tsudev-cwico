//! The release-name ⇄ semver mapping.
//!
//! Releases are named for the day they ship: `tsudev-cwico-v26.8.19`, and
//! `tsudev-cwico-v26.8.19.2` for a second release the same day.
//!
//! That four-component name is not valid semver, and Cargo, the MSI bundler
//! and the updater all require semver. The updater in particular *compares*
//! versions to decide whether a user is out of date, so the encoding has to
//! sort in release order. The name therefore maps to a semver whose patch
//! field carries both the day and the release-of-day counter:
//!
//! ```text
//! patch = day × 100 + n            (n starts at 1)
//!
//! tsudev-cwico-v26.8.19     <->  26.8.1901
//! tsudev-cwico-v26.8.19.2   <->  26.8.1902
//! tsudev-cwico-v26.8.20     <->  26.8.2001
//! tsudev-cwico-v26.9.1      <->  26.9.101
//! ```
//!
//! `tools/version.py` implements the same mapping for the build tooling. Both
//! are tested against the shared vectors in `data/version-cases.json`, so
//! changing one without the other fails a test in the language that was not
//! changed.

use crate::error::{Error, Result};

/// Product name, and the prefix of every release name.
pub const PRODUCT: &str = "tsudev-cwico";

/// At most 99 releases in a day, because the counter shares the patch field
/// with the day number.
pub const MAX_RELEASES_PER_DAY: u32 = 99;

/// A release, decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Release {
    /// Two-digit year, as in the release name: 26 for 2026.
    pub year: u32,
    pub month: u32,
    pub day: u32,
    /// Which release of that day this is, from 1.
    pub counter: u32,
}

impl Release {
    /// Parse a release name. Accepts the full `tsudev-cwico-v26.8.19`, the
    /// short `v26.8.19`, and the bare `26.8.19`, because all three get typed.
    pub fn parse_name(name: &str) -> Result<Self> {
        let trimmed = name.trim();
        let rest = trimmed
            .strip_prefix(&format!("{PRODUCT}-"))
            .unwrap_or(trimmed);
        let rest = rest.strip_prefix('v').unwrap_or(rest);

        let parts: Vec<&str> = rest.split('.').collect();
        if parts.len() < 3 || parts.len() > 4 {
            return Err(Error::other(format!(
                "`{name}` is not a release name; expected {PRODUCT}-vYY.M.D[.N]"
            )));
        }

        let number = |index: usize| -> Result<u32> {
            parts[index]
                .parse::<u32>()
                .map_err(|_| Error::other(format!("`{name}`: `{}` is not a number", parts[index])))
        };

        let year = number(0)?;
        let month = number(1)?;
        let day = number(2)?;
        let counter = if parts.len() == 4 { number(3)? } else { 1 };

        Self::validate(year, month, day, counter, name)?;
        Ok(Self {
            year,
            month,
            day,
            counter,
        })
    }

    /// Decode the semver a build carries back into a release.
    pub fn parse_semver(semver: &str) -> Result<Self> {
        let parts: Vec<&str> = semver.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(Error::other(format!(
                "`{semver}` is not a three-component semver"
            )));
        }
        let number = |index: usize| -> Result<u32> {
            parts[index].parse::<u32>().map_err(|_| {
                Error::other(format!("`{semver}`: `{}` is not a number", parts[index]))
            })
        };

        let year = number(0)?;
        let month = number(1)?;
        let patch = number(2)?;
        let day = patch / 100;
        let counter = patch % 100;

        if counter == 0 {
            return Err(Error::other(format!(
                "`{semver}`: patch {patch} decodes to release-of-day 0, but the \
                 counter starts at 1 - this version was not produced by this scheme"
            )));
        }
        Self::validate(year, month, day, counter, semver)?;
        Ok(Self {
            year,
            month,
            day,
            counter,
        })
    }

    fn validate(year: u32, month: u32, day: u32, counter: u32, source: &str) -> Result<()> {
        let _ = year;
        if !(1..=12).contains(&month) {
            return Err(Error::other(format!(
                "`{source}`: month {month} is out of range"
            )));
        }
        if !(1..=31).contains(&day) {
            return Err(Error::other(format!(
                "`{source}`: day {day} is out of range"
            )));
        }
        if !(1..=MAX_RELEASES_PER_DAY).contains(&counter) {
            return Err(Error::other(format!(
                "`{source}`: release counter {counter} is outside 1..{MAX_RELEASES_PER_DAY}"
            )));
        }
        Ok(())
    }

    /// The semver Cargo, the MSI and the updater use.
    pub fn to_semver(self) -> String {
        format!(
            "{}.{}.{}",
            self.year,
            self.month,
            self.day * 100 + self.counter
        )
    }

    /// The full release name, as users see it.
    pub fn to_name(self) -> String {
        format!("{PRODUCT}-v{}", self.short_name())
    }

    /// Just the version part: `26.8.19` or `26.8.19.2`.
    pub fn short_name(self) -> String {
        if self.counter == 1 {
            format!("{}.{}.{}", self.year, self.month, self.day)
        } else {
            format!("{}.{}.{}.{}", self.year, self.month, self.day, self.counter)
        }
    }

    /// `2026-08-19`, for display next to a release.
    pub fn iso_date(self) -> String {
        format!("20{:02}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Convenience: semver string to the release name users recognise.
///
/// Falls back to the input unchanged when it was not produced by this scheme,
/// so a stray version never turns into an error dialog in the UI.
pub fn name_for_semver(semver: &str) -> String {
    Release::parse_semver(semver)
        .map(Release::to_name)
        .unwrap_or_else(|_| format!("{PRODUCT}-v{semver}"))
}

/// This build's release name.
pub fn current_release_name() -> String {
    name_for_semver(crate::VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Cases {
        round_trip: Vec<RoundTrip>,
        ascending_order: Vec<String>,
        rejected_names: Vec<String>,
        rejected_semvers: Vec<String>,
    }

    #[derive(Deserialize)]
    struct RoundTrip {
        name: String,
        semver: String,
    }

    /// The same vectors `tools/test_version.py` uses. Changing the mapping in
    /// one language without the other fails here.
    fn cases() -> Cases {
        serde_json::from_str(include_str!("../../../data/version-cases.json"))
            .expect("data/version-cases.json must parse")
    }

    #[test]
    fn round_trips_every_shared_case() {
        for case in cases().round_trip {
            let release = Release::parse_name(&case.name).unwrap_or_else(|e| {
                panic!("`{}` should parse: {e}", case.name);
            });
            assert_eq!(release.to_semver(), case.semver, "name -> semver");
            assert_eq!(
                Release::parse_semver(&case.semver).unwrap().to_name(),
                case.name,
                "semver -> name"
            );
        }
    }

    #[test]
    fn semver_ordering_matches_release_order() {
        // The updater compares these to decide whether a user is out of date,
        // so a mapping that round-trips but sorts wrongly would silently stop
        // delivering updates.
        let semvers: Vec<(u32, u32, u32)> = cases()
            .ascending_order
            .iter()
            .map(|name| {
                let r = Release::parse_name(name).unwrap();
                (r.year, r.month, r.day * 100 + r.counter)
            })
            .collect();

        let mut sorted = semvers.clone();
        sorted.sort_unstable();
        assert_eq!(
            semvers, sorted,
            "release order is not preserved by the mapping"
        );
    }

    #[test]
    fn the_tenth_release_of_a_day_sorts_above_the_second() {
        // Naive string comparison gets this wrong: "…19.10" < "…19.2".
        let tenth = Release::parse_name("tsudev-cwico-v26.8.19.10").unwrap();
        let second = Release::parse_name("tsudev-cwico-v26.8.19.2").unwrap();
        assert!(tenth.to_semver() > second.to_semver() || tenth > second);
        assert!(tenth.day * 100 + tenth.counter > second.day * 100 + second.counter);
    }

    #[test]
    fn rejects_what_the_shared_cases_say_it_should() {
        for name in cases().rejected_names {
            assert!(
                Release::parse_name(&name).is_err(),
                "`{name}` should have been rejected"
            );
        }
        for semver in cases().rejected_semvers {
            assert!(
                Release::parse_semver(&semver).is_err(),
                "semver `{semver}` should have been rejected"
            );
        }
    }

    #[test]
    fn accepts_the_short_forms_a_maintainer_would_type() {
        for input in ["tsudev-cwico-v26.8.19", "v26.8.19", "26.8.19"] {
            assert_eq!(Release::parse_name(input).unwrap().to_semver(), "26.8.1901");
        }
    }

    #[test]
    fn an_unrecognised_version_degrades_instead_of_failing() {
        // A stray version must not become an error dialog in the UI.
        assert_eq!(name_for_semver("1.0.0"), "tsudev-cwico-v1.0.0");
    }

    #[test]
    fn this_build_reports_a_valid_release_name() {
        let name = current_release_name();
        assert!(name.starts_with(PRODUCT), "{name}");
        Release::parse_name(&name)
            .unwrap_or_else(|e| panic!("this build's own version does not parse: {e}"));
    }

    #[test]
    fn iso_date_is_rendered_for_display() {
        let r = Release::parse_name("tsudev-cwico-v26.8.19").unwrap();
        assert_eq!(r.iso_date(), "2026-08-19");
    }
}
