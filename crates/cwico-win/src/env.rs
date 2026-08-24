//! Environment-variable expansion.
//!
//! The safety database stores residue paths as `%LOCALAPPDATA%\Vendor\App`.
//! Those have to become real paths before the deletion guard sees them -
//! [`cwico_core::guard`] rejects any path still containing a `%`, precisely so
//! that a failed expansion can never collapse into a short, dangerous path.

/// Expand `%VAR%` occurrences using the process environment.
///
/// A variable that does not exist is left as-is (including its percent
/// signs), so the guard rejects the result rather than silently producing
/// something like `\Vendor\App`.
pub fn expand(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;

    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let name = &after[..end];
                match std::env::var(name) {
                    Ok(value) if !value.is_empty() => out.push_str(&value),
                    // Unknown or empty: keep the literal so the guard trips.
                    _ => {
                        out.push('%');
                        out.push_str(name);
                        out.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                // Unpaired `%`: emit the remainder verbatim.
                out.push('%');
                out.push_str(after);
                return out;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Expand and normalise separators, trimming a trailing backslash.
pub fn expand_path(raw: &str) -> String {
    let expanded = expand(raw.trim());
    let normalised = expanded.replace('/', "\\");
    let trimmed = normalised.trim_end_matches('\\');
    if trimmed.len() < normalised.len() && trimmed.len() <= 2 {
        // `C:` -> keep `C:\`, which the guard will reject as a drive root.
        normalised
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_variables_are_substituted() {
        std::env::set_var("CWICO_TEST_DIR", r"C:\Base");
        assert_eq!(expand(r"%CWICO_TEST_DIR%\App"), r"C:\Base\App");
    }

    #[test]
    fn unknown_variables_survive_so_the_guard_can_reject_them() {
        std::env::remove_var("CWICO_DEFINITELY_UNSET");
        let out = expand(r"%CWICO_DEFINITELY_UNSET%\App");
        assert_eq!(out, r"%CWICO_DEFINITELY_UNSET%\App");
        assert!(
            cwico_core::guard::validate_delete_path(&out).is_err(),
            "an unexpanded path must never be deletable"
        );
    }

    #[test]
    fn empty_variables_are_treated_as_unset() {
        std::env::set_var("CWICO_TEST_EMPTY", "");
        assert!(expand(r"%CWICO_TEST_EMPTY%\App").starts_with('%'));
    }

    #[test]
    fn text_without_variables_is_unchanged() {
        assert_eq!(expand(r"C:\Program Files\App"), r"C:\Program Files\App");
    }

    #[test]
    fn an_unpaired_percent_is_left_alone() {
        assert_eq!(expand(r"C:\100% Free\App"), r"C:\100% Free\App");
    }

    #[test]
    fn multiple_variables_expand() {
        std::env::set_var("CWICO_A", "AA");
        std::env::set_var("CWICO_B", "BB");
        assert_eq!(expand("%CWICO_A%-%CWICO_B%"), "AA-BB");
    }

    #[test]
    fn trailing_separators_are_trimmed() {
        std::env::set_var("CWICO_TEST_DIR", r"C:\Base");
        assert_eq!(expand_path(r"%CWICO_TEST_DIR%\App\"), r"C:\Base\App");
        assert_eq!(expand_path(r"C:/Base/App"), r"C:\Base\App");
    }
}
