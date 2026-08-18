//! Running the vendor's own uninstaller.
//!
//! The parsing half lives in [`crate::cmdline`], which is host-independent so
//! its tests run everywhere. What is left here is the part that genuinely
//! needs an operating system: spawning the uninstaller and waiting for it
//! without letting a hung installer block the run for ever.

use crate::cmdline::{make_silent, parse};
use cwico_core::backend::ExecOutcome;
use cwico_core::{Error, Result};
use std::process::Command;
use std::time::{Duration, Instant};

/// How long to wait for an uninstaller before giving up.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Run an uninstall command and wait for it.
///
/// `prefer_silent` is set when the caller had no `QuietUninstallString` and
/// wants the silent switches inferred where that is safe.
pub fn run(command: &str, prefer_silent: bool, timeout: Duration) -> Result<ExecOutcome> {
    let parsed = parse(command)?;
    let (final_command, silent) = if prefer_silent {
        make_silent(&parsed)
    } else {
        (parsed, false)
    };

    if !silent && prefer_silent {
        tracing::info!(
            program = %final_command.program,
            "unknown installer family: running the vendor command as written, \
             a window may appear"
        );
    }

    let started = Instant::now();
    let mut child = Command::new(&final_command.program)
        .args(&final_command.args)
        .spawn()
        .map_err(|e| Error::UninstallerProcess {
            command: command.to_string(),
            source_msg: format!("could not start `{}`: {e}", final_command.program),
        })?;

    // Poll rather than `wait()` so a hung uninstaller cannot block the run
    // for ever.
    let (exit_code, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let _ = child.kill();
                    break (None, true);
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(e) => {
                return Err(Error::UninstallerProcess {
                    command: command.to_string(),
                    source_msg: format!("waiting on the uninstaller failed: {e}"),
                })
            }
        }
    };

    Ok(ExecOutcome {
        command: format!("{} {}", final_command.program, final_command.args.join(" "))
            .trim_end()
            .to_string(),
        exit_code,
        stdout_tail: String::new(),
        stderr_tail: if timed_out {
            format!("timed out after {} seconds", timeout.as_secs())
        } else {
            String::new()
        },
        timed_out,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}
