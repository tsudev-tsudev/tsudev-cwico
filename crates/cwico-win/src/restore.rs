//! System Restore Point creation.
//!
//! The first line of rollback. Before the engine's first destructive step it
//! calls [`create`], which wraps `SRSetRestorePointW` with an
//! `APPLICATION_UNINSTALL` restore point bracketed by `BEGIN_SYSTEM_CHANGE`
//! and `END_SYSTEM_CHANGE`.
//!
//! Two Windows behaviours matter and are handled explicitly:
//!
//! * System Protection is **off by default on many OEM images**. Calling the
//!   API then fails, and the engine treats that as a run-level abort rather
//!   than proceeding unprotected. [`is_available`] lets the UI warn first.
//! * Windows throttles restore points to one per 24 hours by default
//!   (`SystemRestorePointCreationFrequency`). A throttled call reports success
//!   without creating anything, so [`create`] verifies the returned sequence
//!   number and says plainly when it got nothing.

use crate::registry::{RegKey, RegView};
use cwico_core::backend::RestorePointInfo;
use cwico_core::engine::now_rfc3339;
use cwico_core::{Error, Result};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{HKEY_LOCAL_MACHINE, KEY_READ};
use windows::Win32::System::Restore::{
    SRSetRestorePointW, APPLICATION_UNINSTALL, BEGIN_SYSTEM_CHANGE, END_SYSTEM_CHANGE,
    RESTOREPOINTINFOW, STATEMGRSTATUS,
};

const SR_POLICY_KEY: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\SystemRestore";
const SR_GP_KEY: &str = r"SOFTWARE\Policies\Microsoft\Windows NT\SystemRestore";

/// Description field is a fixed 256-UTF-16-unit array.
const MAX_DESCRIPTION_UNITS: usize = 255;

/// `true` when System Protection looks usable on this machine.
///
/// This is a best-effort pre-flight check against the two registry switches
/// that disable it; the authoritative answer is whether [`create`] succeeds.
pub fn is_available() -> bool {
    let disabled_by_policy = RegKey::open(HKEY_LOCAL_MACHINE, SR_GP_KEY, RegView::Bits64, KEY_READ)
        .ok()
        .and_then(|k| k.u32("DisableSR"))
        .is_some_and(|v| v == 1);
    if disabled_by_policy {
        return false;
    }

    let disabled_locally =
        RegKey::open(HKEY_LOCAL_MACHINE, SR_POLICY_KEY, RegView::Bits64, KEY_READ)
            .ok()
            .and_then(|k| k.u32("DisableSR"))
            .is_some_and(|v| v == 1);
    !disabled_locally
}

/// The configured minimum interval between restore points, in minutes.
/// `0` means Windows creates one on every request.
pub fn creation_frequency_minutes() -> u32 {
    RegKey::open(HKEY_LOCAL_MACHINE, SR_POLICY_KEY, RegView::Bits64, KEY_READ)
        .ok()
        .and_then(|k| k.u32("SystemRestorePointCreationFrequency"))
        // Windows' own default when the value is absent.
        .unwrap_or(1_440)
}

/// Create a restore point named `description`.
///
/// Returns an error when System Protection is off, when the process is not
/// elevated, or when Windows accepted the call but created nothing.
pub fn create(description: &str) -> Result<RestorePointInfo> {
    if !is_available() {
        return Err(Error::RestorePoint(
            "System Protection is turned off for the system drive. Enable it in \
             Settings > System > About > System protection, then try again."
                .into(),
        ));
    }

    let mut info = RESTOREPOINTINFOW {
        dwEventType: BEGIN_SYSTEM_CHANGE,
        dwRestorePtType: APPLICATION_UNINSTALL,
        llSequenceNumber: 0,
        szDescription: [0u16; 256],
    };

    // `RESTOREPOINTINFOW` is `#[repr(packed)]`, so its fields cannot be
    // borrowed. Build the array locally and assign it whole.
    // Truncate on a UTF-16 unit boundary and keep room for the terminator.
    let wide: Vec<u16> = description
        .encode_utf16()
        .take(MAX_DESCRIPTION_UNITS)
        .collect();
    let mut description_buf = [0u16; 256];
    description_buf[..wide.len()].copy_from_slice(&wide);
    info.szDescription = description_buf;

    let mut status = STATEMGRSTATUS::default();
    // SAFETY: both structures are fully initialised and live for the call.
    let ok = unsafe { SRSetRestorePointW(&info, &mut status) };

    if !ok.as_bool() {
        return Err(Error::RestorePoint(describe_failure(status)));
    }
    if status.llSequenceNumber == 0 {
        let minutes = creation_frequency_minutes();
        return Err(Error::RestorePoint(format!(
            "Windows accepted the request but created no restore point. It throttles \
             creation to one every {minutes} minutes; the most recent point is still \
             recent enough that this run would not be covered by a new one."
        )));
    }

    let sequence = status.llSequenceNumber;

    // Close the change bracket so the point is not left "in progress".
    let end = RESTOREPOINTINFOW {
        dwEventType: END_SYSTEM_CHANGE,
        dwRestorePtType: APPLICATION_UNINSTALL,
        llSequenceNumber: sequence,
        szDescription: description_buf,
    };
    let mut end_status = STATEMGRSTATUS::default();
    // SAFETY: same invariants as above.
    let end_ok = unsafe { SRSetRestorePointW(&end, &mut end_status) };
    if !end_ok.as_bool() {
        // Copy out of the packed struct before logging: a field of a packed
        // struct cannot be borrowed.
        let code = end_status.nStatus.0;
        // The point exists; only the bracket failed to close. Worth a warning,
        // not worth aborting a run that is now protected.
        tracing::warn!(sequence, code, "could not close the system-change bracket");
    }

    tracing::info!(sequence, description, "system restore point created");
    Ok(RestorePointInfo {
        sequence_number: sequence,
        description: description.to_string(),
        created_at: now_rfc3339(),
    })
}

fn describe_failure(status: STATEMGRSTATUS) -> String {
    // Copied out of the packed struct; borrowing the field is undefined
    // behaviour even when the reference is never dereferenced.
    let code = status.nStatus;
    let hint = match code.0 {
        // ERROR_ACCESS_DENIED
        5 => " (the process needs to run as Administrator)",
        // ERROR_SERVICE_DISABLED
        1058 => " (the Volume Shadow Copy or System Restore service is disabled)",
        // ERROR_INVALID_FUNCTION — typical on systems with protection off
        1 => " (System Protection is not enabled for the system drive)",
        _ if code == ERROR_SUCCESS => " (the call reported failure with no error code)",
        _ => "",
    };
    format!("SRSetRestorePointW failed with error {}{hint}", code.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_description_is_truncated_on_a_unit_boundary() {
        // The struct field is a fixed 256-unit array; overrunning it is a
        // buffer overflow, so make sure the truncation logic is exercised.
        let long = "x".repeat(1_000);
        let wide: Vec<u16> = long.encode_utf16().take(MAX_DESCRIPTION_UNITS).collect();
        assert_eq!(wide.len(), MAX_DESCRIPTION_UNITS);
        assert!(wide.len() < 256, "must leave room for the NUL terminator");
    }

    #[test]
    fn multibyte_descriptions_do_not_overflow_the_buffer() {
        let vietnamese = "Gỡ bỏ phần mềm không cần thiết ".repeat(30);
        let wide: Vec<u16> = vietnamese
            .encode_utf16()
            .take(MAX_DESCRIPTION_UNITS)
            .collect();
        assert!(wide.len() <= MAX_DESCRIPTION_UNITS);
    }

    #[test]
    fn failure_messages_name_the_likely_cause() {
        let denied = STATEMGRSTATUS {
            nStatus: windows::Win32::Foundation::WIN32_ERROR(5),
            llSequenceNumber: 0,
        };
        assert!(describe_failure(denied).contains("Administrator"));

        let disabled = STATEMGRSTATUS {
            nStatus: windows::Win32::Foundation::WIN32_ERROR(1058),
            llSequenceNumber: 0,
        };
        assert!(describe_failure(disabled).contains("disabled"));
    }
}
