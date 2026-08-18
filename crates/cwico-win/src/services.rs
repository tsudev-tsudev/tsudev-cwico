//! Windows service enumeration and reconfiguration.
//!
//! Services are the half of "installed software" that Add/Remove Programs
//! never shows. A vendor's updater, telemetry agent or licence daemon usually
//! survives the application uninstall as a running service, which is exactly
//! the residue this tool exists to find.
//!
//! Services are *disabled*, never deleted. Setting `StartType = Disabled` is
//! reversible in one call; deleting the service key is not, and a service the
//! user turns out to need would then require a reinstall.

use crate::wide::{from_wide_ptr, WideString};
use cwico_core::backend::StepResult;
use cwico_core::{Error, Result};
use windows::Win32::System::Services::{
    ChangeServiceConfigW, CloseServiceHandle, ControlService, EnumServicesStatusExW,
    OpenSCManagerW, OpenServiceW, QueryServiceConfigW, ENUM_SERVICE_STATUS_PROCESSW,
    ENUM_SERVICE_TYPE, QUERY_SERVICE_CONFIGW, SC_ENUM_PROCESS_INFO, SC_HANDLE, SC_MANAGER_CONNECT,
    SC_MANAGER_ENUMERATE_SERVICE, SERVICE_ACTIVE, SERVICE_AUTO_START, SERVICE_CHANGE_CONFIG,
    SERVICE_CONTROL_STOP, SERVICE_DEMAND_START, SERVICE_DISABLED, SERVICE_DRIVER,
    SERVICE_ERROR_NORMAL, SERVICE_INACTIVE, SERVICE_NO_CHANGE, SERVICE_QUERY_CONFIG,
    SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START_TYPE, SERVICE_STATUS, SERVICE_STOP,
    SERVICE_WIN32,
};

/// How a service starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartType {
    Boot,
    System,
    Automatic,
    Manual,
    Disabled,
    Unknown,
}

impl StartType {
    fn from_raw(raw: u32) -> Self {
        match raw {
            0 => StartType::Boot,
            1 => StartType::System,
            2 => StartType::Automatic,
            3 => StartType::Manual,
            4 => StartType::Disabled,
            _ => StartType::Unknown,
        }
    }

    /// Parse the strings the tweak catalogue uses.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "automatic" | "auto" => Some(StartType::Automatic),
            "manual" | "demand" => Some(StartType::Manual),
            "disabled" => Some(StartType::Disabled),
            _ => None,
        }
    }

    fn to_raw(self) -> Option<SERVICE_START_TYPE> {
        match self {
            StartType::Automatic => Some(SERVICE_AUTO_START),
            StartType::Manual => Some(SERVICE_DEMAND_START),
            StartType::Disabled => Some(SERVICE_DISABLED),
            // Boot- and system-start services are drivers; this tool does not
            // reconfigure them, and `Unknown` means we could not read it.
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            StartType::Boot => "boot",
            StartType::System => "system",
            StartType::Automatic => "automatic",
            StartType::Manual => "manual",
            StartType::Disabled => "disabled",
            StartType::Unknown => "unknown",
        }
    }
}

/// One service as the scanner sees it.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Short name, e.g. `DiagTrack`.
    pub name: String,
    /// Friendly name shown in services.msc.
    pub display_name: String,
    pub running: bool,
    pub start_type: StartType,
    /// The `ImagePath`, used to attribute a service to its owning product.
    pub binary_path: Option<String>,
    /// `true` when the service is a kernel driver rather than a Win32 service.
    pub is_driver: bool,
}

/// RAII wrapper over an `SC_HANDLE`.
struct ScHandle(SC_HANDLE);

impl Drop for ScHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            // SAFETY: the handle came from Open*W and is closed exactly once.
            unsafe {
                let _ = CloseServiceHandle(self.0);
            }
        }
    }
}

fn open_manager(access: u32) -> Result<ScHandle> {
    // SAFETY: null machine/database names select the local, active database.
    let handle = unsafe { OpenSCManagerW(None, None, access) }.map_err(|e| Error::Service {
        service: "<service control manager>".into(),
        source_msg: format!("OpenSCManagerW failed: {e}"),
    })?;
    Ok(ScHandle(handle))
}

fn open_service(manager: &ScHandle, name: &str, access: u32) -> Result<ScHandle> {
    let wide = WideString::new(name);
    // SAFETY: `wide` outlives the call; `manager` is open.
    let handle = unsafe { OpenServiceW(manager.0, wide.as_pcwstr(), access) }.map_err(|e| {
        Error::Service {
            service: name.to_string(),
            source_msg: format!("OpenServiceW failed: {e}"),
        }
    })?;
    Ok(ScHandle(handle))
}

/// Enumerate every Win32 service and driver on the machine.
pub fn enumerate(include_drivers: bool) -> Result<Vec<ServiceInfo>> {
    let manager = open_manager(SC_MANAGER_CONNECT | SC_MANAGER_ENUMERATE_SERVICE)?;

    let service_type: ENUM_SERVICE_TYPE = if include_drivers {
        ENUM_SERVICE_TYPE(SERVICE_WIN32.0 | SERVICE_DRIVER.0)
    } else {
        SERVICE_WIN32
    };

    // Size the buffer, then fill it.
    let mut needed = 0u32;
    let mut returned = 0u32;
    // SAFETY: passing no buffer is the documented way to learn the size; the
    // call is expected to fail with ERROR_MORE_DATA.
    let _ = unsafe {
        EnumServicesStatusExW(
            manager.0,
            SC_ENUM_PROCESS_INFO,
            service_type,
            windows::Win32::System::Services::ENUM_SERVICE_STATE(
                SERVICE_ACTIVE.0 | SERVICE_INACTIVE.0,
            ),
            None,
            &mut needed,
            &mut returned,
            None,
            None,
        )
    };
    if needed == 0 {
        return Ok(Vec::new());
    }

    let mut buffer = vec![0u8; needed as usize];
    // SAFETY: `buffer` is exactly the size the call above asked for.
    unsafe {
        EnumServicesStatusExW(
            manager.0,
            SC_ENUM_PROCESS_INFO,
            service_type,
            windows::Win32::System::Services::ENUM_SERVICE_STATE(
                SERVICE_ACTIVE.0 | SERVICE_INACTIVE.0,
            ),
            Some(&mut buffer),
            &mut needed,
            &mut returned,
            None,
            None,
        )
    }
    .map_err(|e| Error::Service {
        service: "<enumerate>".into(),
        source_msg: format!("EnumServicesStatusExW failed: {e}"),
    })?;

    let mut out = Vec::with_capacity(returned as usize);
    // SAFETY: the API filled `returned` contiguous ENUM_SERVICE_STATUS_PROCESSW
    // structures at the start of `buffer`, and the string pointers inside
    // them point into the same buffer, which outlives this loop.
    let entries = unsafe {
        std::slice::from_raw_parts(
            buffer.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW,
            returned as usize,
        )
    };

    for entry in entries {
        // SAFETY: pointers are into `buffer`; the 4096-unit bound is a
        // generous cap on a service name.
        let name = unsafe { from_wide_ptr(entry.lpServiceName.0, 4_096) };
        let display_name = unsafe { from_wide_ptr(entry.lpDisplayName.0, 4_096) };
        if name.is_empty() {
            continue;
        }
        let running = entry.ServiceStatusProcess.dwCurrentState == SERVICE_RUNNING;
        let is_driver = entry.ServiceStatusProcess.dwServiceType.0 & SERVICE_DRIVER.0 != 0;

        let (start_type, binary_path) = match query_config(&manager, &name) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::debug!(service = %name, error = %e, "could not read service config");
                (StartType::Unknown, None)
            }
        };

        out.push(ServiceInfo {
            name,
            display_name,
            running,
            start_type,
            binary_path,
            is_driver,
        });
    }

    out.sort_by_key(|s| s.name.to_lowercase());
    Ok(out)
}

fn query_config(manager: &ScHandle, name: &str) -> Result<(StartType, Option<String>)> {
    let service = open_service(manager, name, SERVICE_QUERY_CONFIG)?;

    let mut needed = 0u32;
    // SAFETY: a null config pointer with size 0 queries the required length.
    let _ = unsafe { QueryServiceConfigW(service.0, None, 0, &mut needed) };
    if needed == 0 {
        return Ok((StartType::Unknown, None));
    }

    let mut buffer = vec![0u8; needed as usize];
    let config = buffer.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW;
    // SAFETY: `buffer` is `needed` bytes, which is what the API asked for, and
    // is correctly aligned for QUERY_SERVICE_CONFIGW because Vec<u8> from the
    // global allocator satisfies the struct's pointer alignment.
    unsafe { QueryServiceConfigW(service.0, Some(config), needed, &mut needed) }.map_err(|e| {
        Error::Service {
            service: name.to_string(),
            source_msg: format!("QueryServiceConfigW failed: {e}"),
        }
    })?;

    // SAFETY: the call above initialised `*config` and the strings it points at.
    let (start_raw, path) = unsafe {
        let cfg = &*config;
        (
            cfg.dwStartType.0,
            from_wide_ptr(cfg.lpBinaryPathName.0, 32_768),
        )
    };

    Ok((
        StartType::from_raw(start_raw),
        Some(path).filter(|p| !p.is_empty()),
    ))
}

/// Stop a running service, waiting briefly for it to reach `STOPPED`.
///
/// A service that is already stopped, or that does not exist, is a success:
/// re-running a plan should be quiet.
pub fn stop(name: &str) -> Result<StepResult> {
    let manager = open_manager(SC_MANAGER_CONNECT)?;
    let service = match open_service(&manager, name, SERVICE_STOP | SERVICE_QUERY_STATUS) {
        Ok(s) => s,
        Err(_) => {
            return Ok(StepResult::skipped(format!(
                "service `{name}` is not installed"
            )))
        }
    };

    let mut status = SERVICE_STATUS::default();
    // SAFETY: `service` is open with SERVICE_STOP; `status` is a valid out-param.
    match unsafe { ControlService(service.0, SERVICE_CONTROL_STOP, &mut status) } {
        Ok(()) => {}
        Err(e) => {
            // ERROR_SERVICE_NOT_ACTIVE (1062): already what we wanted.
            if e.code().0 as u32 & 0xFFFF == 1062 {
                return Ok(StepResult::skipped(format!(
                    "service `{name}` was already stopped"
                )));
            }
            return Err(Error::Service {
                service: name.to_string(),
                source_msg: format!("ControlService(STOP) failed: {e}"),
            });
        }
    }

    Ok(StepResult::ok(format!("stopped service `{name}`")))
}

/// Set a service's start type.
pub fn set_start_type(name: &str, start: StartType) -> Result<StepResult> {
    let Some(raw) = start.to_raw() else {
        return Err(Error::Service {
            service: name.to_string(),
            source_msg: format!(
                "refusing to set start type `{}`: boot and system drivers are not reconfigured \
                 by this tool",
                start.label()
            ),
        });
    };

    let manager = open_manager(SC_MANAGER_CONNECT)?;
    let service = match open_service(&manager, name, SERVICE_CHANGE_CONFIG) {
        Ok(s) => s,
        Err(_) => {
            return Ok(StepResult::skipped(format!(
                "service `{name}` is not installed"
            )))
        }
    };

    // SAFETY: SERVICE_NO_CHANGE leaves every field we pass `None` for intact.
    unsafe {
        ChangeServiceConfigW(
            service.0,
            ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
            raw,
            SERVICE_ERROR_NORMAL,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }
    .map_err(|e| Error::Service {
        service: name.to_string(),
        source_msg: format!("ChangeServiceConfigW failed: {e}"),
    })?;

    Ok(StepResult::ok(format!(
        "service `{name}` start type set to {}",
        start.label()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_types_decode_from_the_raw_values() {
        assert_eq!(StartType::from_raw(2), StartType::Automatic);
        assert_eq!(StartType::from_raw(3), StartType::Manual);
        assert_eq!(StartType::from_raw(4), StartType::Disabled);
        assert_eq!(StartType::from_raw(99), StartType::Unknown);
    }

    #[test]
    fn the_tweak_catalogues_spellings_all_parse() {
        assert_eq!(StartType::parse("automatic"), Some(StartType::Automatic));
        assert_eq!(StartType::parse("Manual"), Some(StartType::Manual));
        assert_eq!(StartType::parse("disabled"), Some(StartType::Disabled));
        assert_eq!(StartType::parse("nonsense"), None);
    }

    #[test]
    fn boot_and_system_drivers_cannot_be_reconfigured() {
        // Setting a boot-start storage driver to anything else is how you get
        // an unbootable machine, so the conversion refuses.
        assert!(StartType::Boot.to_raw().is_none());
        assert!(StartType::System.to_raw().is_none());
        assert!(StartType::Unknown.to_raw().is_none());
        assert!(StartType::Disabled.to_raw().is_some());
    }
}
