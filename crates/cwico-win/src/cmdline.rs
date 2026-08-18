//! Parsing `UninstallString`, and choosing silent switches.
//!
//! `UninstallString` is free-form text written by whoever built the installer,
//! and it arrives in every shape imaginable:
//!
//! ```text
//! "C:\Program Files\App\uninst.exe" /S
//! C:\Windows\SysWOW64\OneDriveSetup.exe /uninstall
//! MsiExec.exe /I{90160000-008C-0000-1000-0000000FF1CE}
//! C:\Program Files\App\unins000.exe
//! ```
//!
//! Parsing it wrong means launching the wrong executable, so this module is
//! pure `std` with no Windows dependency — it compiles and its tests run on
//! any host, which is what lets CI cover it.
//!
//! Silent operation is only *inferred* for installer families whose switches
//! are documented and stable — MSI, Inno Setup, NSIS. For anything else the
//! command runs exactly as the vendor wrote it, because guessing a flag at an
//! unknown installer is how you end up with a repair instead of a removal.

use cwico_core::{Error, Result};

/// The installer family a command belongs to, which decides the silent flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerKind {
    /// Windows Installer. `/qn /norestart` is documented and safe.
    Msi,
    /// Inno Setup's `unins###.exe`.
    InnoSetup,
    /// NSIS uninstallers, which accept `/S`.
    Nsis,
    /// Anything else: run verbatim.
    Unknown,
}

/// A parsed uninstall command.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedCommand {
    pub program: String,
    pub args: Vec<String>,
}

/// Split a command line the way `CommandLineToArgvW` does: double quotes group
/// a token, and everything else splits on whitespace.
pub fn tokenize(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut has_token = false;

    for ch in command.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                has_token = true;
            }
            c if c.is_whitespace() && !in_quotes => {
                if has_token {
                    tokens.push(std::mem::take(&mut current));
                    has_token = false;
                }
            }
            c => {
                current.push(c);
                has_token = true;
            }
        }
    }
    if has_token {
        tokens.push(current);
    }
    tokens
}

/// Executable suffixes used to find where an unquoted program path ends.
const PROGRAM_SUFFIXES: &[&str] = &[".exe", ".com", ".bat", ".cmd", ".msi"];

/// Find the end of an unquoted program path.
///
/// `C:\Program Files\App\unins000.exe /SILENT` has a space inside the path
/// and no quotes — extremely common in real `UninstallString` values. Splitting
/// on the first space yields `C:\Program`, which is not a program. Instead,
/// scan for the first executable suffix that is followed by whitespace or the
/// end of the string.
fn unquoted_program_end(command: &str) -> Option<usize> {
    let lower = command.to_ascii_lowercase();
    let mut best: Option<usize> = None;

    for suffix in PROGRAM_SUFFIXES {
        let mut from = 0usize;
        while let Some(found) = lower[from..].find(suffix) {
            let end = from + found + suffix.len();
            let boundary = command[end..]
                .chars()
                .next()
                .is_none_or(char::is_whitespace);
            if boundary {
                best = Some(match best {
                    // The *earliest* match wins: a later `.exe` belongs to an
                    // argument, not to the program.
                    Some(previous) => previous.min(end),
                    None => end,
                });
                break;
            }
            from = end;
        }
    }
    best
}

/// Parse an `UninstallString` into a program and its arguments.
pub fn parse(command: &str) -> Result<ParsedCommand> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return Err(Error::UninstallerProcess {
            command: command.to_string(),
            source_msg: "the uninstall command is empty".into(),
        });
    }

    // A quoted program is unambiguous: the tokenizer already handles it.
    if !trimmed.starts_with('"') {
        if let Some(end) = unquoted_program_end(trimmed) {
            let program = trimmed[..end].to_string();
            let args = tokenize(&trimmed[end..]);
            return Ok(ParsedCommand { program, args });
        }
    }

    let tokens = tokenize(trimmed);
    let mut iter = tokens.into_iter();
    let program = iter.next().ok_or_else(|| Error::UninstallerProcess {
        command: command.to_string(),
        source_msg: "the uninstall command is empty".into(),
    })?;
    Ok(ParsedCommand {
        program,
        args: iter.collect(),
    })
}

/// Identify the installer family from a parsed command.
pub fn classify(parsed: &ParsedCommand) -> InstallerKind {
    let leaf = parsed
        .program
        .rsplit(['\\', '/'])
        .next()
        .unwrap_or(&parsed.program)
        .to_ascii_lowercase();

    if leaf.starts_with("msiexec") {
        return InstallerKind::Msi;
    }

    // NSIS first. `Uninstall.exe` also starts with "unins", so a loose Inno
    // prefix check would claim it and hand an NSIS uninstaller Inno's
    // `/VERYSILENT` flag — which it does not understand, so it would open a
    // window instead of running silently.
    if matches!(
        leaf.as_str(),
        "uninstall.exe" | "uninst.exe" | "uninst32.exe" | "uninstaller.exe"
    ) {
        return InstallerKind::Nsis;
    }

    // Inno Setup writes `unins000.exe`, `unins001.exe`, … — always `unins`
    // followed by digits.
    if let Some(rest) = leaf.strip_prefix("unins") {
        if let Some(digits) = rest.strip_suffix(".exe") {
            if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
                return InstallerKind::InnoSetup;
            }
        }
    }

    InstallerKind::Unknown
}

/// Add the documented silent switches for a known installer family.
///
/// Returns the command unchanged for [`InstallerKind::Unknown`], and reports
/// whether it managed to make the run silent so the caller can tell the user
/// that a window may appear.
pub fn make_silent(parsed: &ParsedCommand) -> (ParsedCommand, bool) {
    let mut out = parsed.clone();
    let has = |args: &[String], needle: &str| args.iter().any(|a| a.eq_ignore_ascii_case(needle));

    match classify(parsed) {
        InstallerKind::Msi => {
            // `/I{GUID}` means install; the uninstall verb is `/X{GUID}`.
            for arg in &mut out.args {
                if arg.len() > 2 && (arg.starts_with("/I") || arg.starts_with("/i")) {
                    let rest = &arg[2..];
                    if rest.starts_with('{') {
                        *arg = format!("/X{rest}");
                    }
                }
            }
            if !has(&out.args, "/qn") && !has(&out.args, "/quiet") {
                out.args.push("/qn".into());
            }
            if !has(&out.args, "/norestart") {
                out.args.push("/norestart".into());
            }
            (out, true)
        }
        InstallerKind::InnoSetup => {
            for flag in ["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"] {
                if !has(&out.args, flag) {
                    out.args.push(flag.into());
                }
            }
            (out, true)
        }
        InstallerKind::Nsis => {
            // NSIS is case-sensitive here: `/S` is silent, `/s` is not.
            if !out.args.iter().any(|a| a == "/S") {
                out.args.push("/S".into());
            }
            (out, true)
        }
        InstallerKind::Unknown => (out, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quoted_path_with_spaces_stays_one_token() {
        let parsed = parse(r#""C:\Program Files\App\uninst.exe" /S"#).unwrap();
        assert_eq!(parsed.program, r"C:\Program Files\App\uninst.exe");
        assert_eq!(parsed.args, vec!["/S"]);
    }

    #[test]
    fn an_unquoted_path_without_spaces_parses() {
        let parsed = parse(r"C:\Windows\SysWOW64\OneDriveSetup.exe /uninstall").unwrap();
        assert_eq!(parsed.program, r"C:\Windows\SysWOW64\OneDriveSetup.exe");
        assert_eq!(parsed.args, vec!["/uninstall"]);
    }

    #[test]
    fn multiple_arguments_are_preserved_in_order() {
        let parsed = parse(r#""C:\a b\s.exe" --uninstall --system-level --verbose"#).unwrap();
        assert_eq!(
            parsed.args,
            vec!["--uninstall", "--system-level", "--verbose"]
        );
    }

    #[test]
    fn an_empty_command_is_an_error_not_a_panic() {
        assert!(parse("").is_err());
        assert!(parse("   ").is_err());
    }

    #[test]
    fn installer_families_are_recognised() {
        let cases = [
            ("MsiExec.exe /I{GUID}", InstallerKind::Msi),
            (
                r"C:\Windows\System32\msiexec.exe /X{GUID}",
                InstallerKind::Msi,
            ),
            (
                r"C:\Program Files\App\unins000.exe",
                InstallerKind::InnoSetup,
            ),
            (r"C:\Program Files\App\Uninstall.exe", InstallerKind::Nsis),
            (r"C:\App\setup.exe --uninstall", InstallerKind::Unknown),
        ];
        for (command, expected) in cases {
            assert_eq!(classify(&parse(command).unwrap()), expected, "{command}");
        }
    }

    #[test]
    fn msi_install_verb_is_flipped_to_uninstall() {
        // Leaving `/I` in place would *repair* the product rather than remove
        // it — a silent no-op the user would read as a failed uninstall.
        let (silent, ok) = make_silent(&parse("MsiExec.exe /I{90160000-008C}").unwrap());
        assert!(ok);
        assert!(silent.args.contains(&"/X{90160000-008C}".to_string()));
        assert!(!silent.args.iter().any(|a| a.starts_with("/I")));
        assert!(silent.args.contains(&"/qn".to_string()));
        assert!(silent.args.contains(&"/norestart".to_string()));
    }

    #[test]
    fn an_existing_uninstall_verb_is_left_alone() {
        let (silent, _) = make_silent(&parse("MsiExec.exe /X{GUID}").unwrap());
        assert!(silent.args.contains(&"/X{GUID}".to_string()));
        assert_eq!(
            silent.args.iter().filter(|a| a.starts_with("/X")).count(),
            1
        );
    }

    #[test]
    fn silent_flags_are_not_duplicated() {
        let (silent, _) = make_silent(&parse("MsiExec.exe /X{GUID} /qn /norestart").unwrap());
        assert_eq!(silent.args.iter().filter(|a| *a == "/qn").count(), 1);
        assert_eq!(silent.args.iter().filter(|a| *a == "/norestart").count(), 1);
    }

    #[test]
    fn inno_setup_gets_its_documented_switches() {
        let (silent, ok) = make_silent(&parse(r"C:\App\unins000.exe").unwrap());
        assert!(ok);
        assert!(silent.args.contains(&"/VERYSILENT".to_string()));
        assert!(silent.args.contains(&"/SUPPRESSMSGBOXES".to_string()));
    }

    #[test]
    fn nsis_gets_an_uppercase_switch() {
        // NSIS treats `/s` and `/S` differently; only `/S` is silent.
        let (silent, ok) = make_silent(&parse(r"C:\App\Uninstall.exe").unwrap());
        assert!(ok);
        assert!(silent.args.contains(&"/S".to_string()));
    }

    #[test]
    fn an_unknown_installer_is_never_given_invented_flags() {
        let original = parse(r#""C:\App\setup.exe" --uninstall"#).unwrap();
        let (result, ok) = make_silent(&original);
        assert!(!ok, "must report that it could not make this silent");
        assert_eq!(result, original, "arguments must not be invented");
    }

    #[test]
    fn an_unquoted_program_path_containing_spaces_is_not_split() {
        // Real `UninstallString` values from Inno Setup and NSIS are usually
        // unquoted, and Program Files has a space in it. Splitting on the
        // first space yields `C:\Program`, which launches nothing.
        let parsed = parse(r"C:\Program Files\App\unins000.exe /SILENT").unwrap();
        assert_eq!(parsed.program, r"C:\Program Files\App\unins000.exe");
        assert_eq!(parsed.args, vec!["/SILENT"]);
        assert_eq!(classify(&parsed), InstallerKind::InnoSetup);
    }

    #[test]
    fn an_unquoted_path_with_no_arguments_parses_whole() {
        let parsed = parse(r"C:\Program Files (x86)\Vendor\Product\uninstall.exe").unwrap();
        assert_eq!(
            parsed.program,
            r"C:\Program Files (x86)\Vendor\Product\uninstall.exe"
        );
        assert!(parsed.args.is_empty());
    }

    #[test]
    fn an_exe_inside_an_argument_does_not_end_the_program_path() {
        let parsed = parse(r"C:\Tools\runner.exe --target C:\App\thing.exe").unwrap();
        assert_eq!(parsed.program, r"C:\Tools\runner.exe");
        assert_eq!(parsed.args, vec!["--target", r"C:\App\thing.exe"]);
    }

    #[test]
    fn nsis_uninstall_exe_is_not_mistaken_for_inno_setup() {
        // `Uninstall.exe` starts with "unins". Inno's `/VERYSILENT` means
        // nothing to NSIS, so misclassifying here shows the user a window
        // during what was supposed to be a silent removal.
        let nsis = parse(r"C:\App\Uninstall.exe").unwrap();
        assert_eq!(classify(&nsis), InstallerKind::Nsis);

        let (silent, _) = make_silent(&nsis);
        assert!(silent.args.contains(&"/S".to_string()));
        assert!(!silent.args.contains(&"/VERYSILENT".to_string()));
    }

    #[test]
    fn inno_setup_needs_digits_after_unins() {
        assert_eq!(
            classify(&parse(r"C:\App\unins000.exe").unwrap()),
            InstallerKind::InnoSetup
        );
        assert_eq!(
            classify(&parse(r"C:\App\unins042.exe").unwrap()),
            InstallerKind::InnoSetup
        );
        // Not Inno: no digits.
        assert_eq!(
            classify(&parse(r"C:\App\uninstaller.exe").unwrap()),
            InstallerKind::Nsis
        );
    }

    #[test]
    fn tokenizer_handles_quotes_inside_arguments() {
        let tokens = tokenize(r#"a.exe "arg with spaces" plain"#);
        assert_eq!(tokens, vec!["a.exe", "arg with spaces", "plain"]);
    }

    #[test]
    fn tokenizer_keeps_an_empty_quoted_argument() {
        let tokens = tokenize(r#"a.exe "" x"#);
        assert_eq!(tokens, vec!["a.exe", "", "x"]);
    }
}
