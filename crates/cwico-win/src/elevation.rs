//! Privilege detection.
//!
//! Almost nothing this tool does works from a standard-user token: HKLM is
//! read-only, the service control manager refuses `SERVICE_CHANGE_CONFIG`,
//! and `SRSetRestorePointW` fails outright. The engine checks this before it
//! starts a real run, and the UI shows a relaunch-as-administrator prompt.

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

/// `true` when the current process runs with an elevated token.
pub fn is_elevated() -> bool {
    // SAFETY: every handle opened here is closed on both paths, and
    // `GetTokenInformation` is given a correctly sized `TOKEN_ELEVATION`.
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut core::ffi::c_void),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        )
        .is_ok();

        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}
