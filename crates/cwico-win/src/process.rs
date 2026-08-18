//! Process enumeration and termination.
//!
//! Step one of the uninstall flow. An installer that cannot replace or delete
//! a file because the application is still running usually fails halfway,
//! leaving a half-removed product — so the running processes go first.
//!
//! The hard rule here is the protected list in [`crate::protected`]:
//! `svchost.exe` hosts a dozen unrelated services, and `lsass.exe` or
//! `csrss.exe` take the machine down with them. No safety rule and no user
//! selection can cause those to be terminated.

use crate::protected::is_protected;
use crate::wide::from_wide;
use cwico_core::backend::KilledProcess;
use cwico_core::{Error, Result};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, TerminateProcess, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};

/// A running process.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    /// Image file name, e.g. `OneDrive.exe`.
    pub name: String,
    /// Full path, when it could be read (needs the process to be openable).
    pub path: Option<String>,
}

/// Snapshot every running process.
pub fn enumerate() -> Result<Vec<ProcessInfo>> {
    // SAFETY: the snapshot handle is closed on every path below.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(|e| Error::other(format!("CreateToolhelp32Snapshot failed: {e}")))?;

    let mut out = Vec::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    // SAFETY: `entry.dwSize` is set as the API requires; `snapshot` is valid.
    let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while ok {
        let name = from_wide(&entry.szExeFile);
        if !name.is_empty() {
            out.push(ProcessInfo {
                pid: entry.th32ProcessID,
                path: image_path(entry.th32ProcessID),
                name,
            });
        }
        // SAFETY: same invariants as the Process32FirstW call.
        ok = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }

    // SAFETY: `snapshot` came from CreateToolhelp32Snapshot and is closed once.
    unsafe {
        let _ = CloseHandle(snapshot);
    }
    Ok(out)
}

/// Full image path for a pid, or `None` when the process cannot be opened
/// (protected processes and processes owned by other users).
fn image_path(pid: u32) -> Option<String> {
    // SAFETY: the handle is closed before returning on both paths.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 32_768];
        let mut len = buf.len() as u32;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(handle);
        result.ok()?;
        Some(from_wide(&buf[..len as usize]))
    }
}

/// Terminate every running process whose image name matches one of
/// `executables` (case-insensitive, file-name only).
///
/// Protected images are skipped and logged, never terminated. A process that
/// exits between the snapshot and the `OpenProcess` is not an error — it is
/// exactly the outcome we wanted.
pub fn kill_matching(executables: &[String]) -> Result<Vec<KilledProcess>> {
    let wanted: Vec<String> = executables
        .iter()
        .map(|e| {
            e.rsplit(['\\', '/'])
                .next()
                .unwrap_or(e)
                .to_ascii_lowercase()
        })
        .filter(|e| !e.is_empty())
        .collect();
    if wanted.is_empty() {
        return Ok(Vec::new());
    }

    let mut killed = Vec::new();
    for proc in enumerate()? {
        let leaf = proc.name.to_ascii_lowercase();
        if !wanted.contains(&leaf) {
            continue;
        }
        if is_protected(&leaf) {
            tracing::warn!(
                process = %proc.name,
                pid = proc.pid,
                "refusing to terminate a protected process; \
                 stop the owning service instead"
            );
            continue;
        }

        // SAFETY: the handle is closed on every path.
        let terminated = unsafe {
            match OpenProcess(PROCESS_TERMINATE, false, proc.pid) {
                Ok(handle) => {
                    let result = TerminateProcess(handle, 0);
                    let _ = CloseHandle(handle);
                    result.is_ok()
                }
                // Already exited, or we lack rights. Neither is fatal: the
                // uninstaller will tell us if the file is still locked.
                Err(e) => {
                    tracing::debug!(
                        process = %proc.name,
                        pid = proc.pid,
                        error = %e,
                        "could not open the process for termination"
                    );
                    false
                }
            }
        };

        if terminated {
            tracing::info!(process = %proc.name, pid = proc.pid, "terminated");
            killed.push(KilledProcess {
                pid: proc.pid,
                name: proc.name,
                path: proc.path,
            });
        }
    }

    Ok(killed)
}

/// Wait until no process matching `executables` is running, or the deadline
/// passes. Used after termination so the uninstaller does not race the
/// kernel closing file handles.
pub fn wait_until_gone(executables: &[String], timeout: std::time::Duration) -> bool {
    let wanted: Vec<String> = executables.iter().map(|e| e.to_ascii_lowercase()).collect();
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let still_running = enumerate()
            .map(|list| {
                list.iter()
                    .any(|p| wanted.contains(&p.name.to_ascii_lowercase()))
            })
            .unwrap_or(false);
        if !still_running {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(std::time::Duration::from_millis(150));
    }
}
