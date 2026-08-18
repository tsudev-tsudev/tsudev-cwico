//! Scheduled task enumeration and enable/disable, via the Task Scheduler 2.0
//! COM interfaces.
//!
//! Scheduled tasks are the third place software hides. An application can be
//! uninstalled and still leave a task that re-downloads its updater every
//! morning, and the telemetry pipeline is almost entirely tasks rather than
//! services.
//!
//! As with services, tasks are **disabled, not deleted**. `SetEnabled(false)`
//! is a single reversible call; `DeleteTask` is not, and Windows recreates
//! some of its own tasks on the next servicing operation anyway.

use crate::naming::split_task_path;
use cwico_core::backend::StepResult;
use cwico_core::{Error, Result};
use windows::core::{BSTR, GUID};
use windows::Win32::Foundation::{VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
};
use windows::Win32::System::TaskScheduler::{
    ITaskService, TaskScheduler, TASK_STATE_DISABLED, TASK_STATE_RUNNING,
};
use windows::Win32::System::Variant::VARIANT;

/// One scheduled task.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Full path, e.g. `\Microsoft\Windows\Defrag\ScheduledDefrag`.
    pub path: String,
    /// Leaf name.
    pub name: String,
    pub enabled: bool,
    pub running: bool,
    /// The task XML, which carries the command it runs. Only fetched when
    /// requested, because it is large and there are hundreds of tasks.
    pub xml: Option<String>,
}

/// COM apartment guard.
///
/// The Task Scheduler interfaces are apartment-threaded, so every thread that
/// touches them has to initialise and uninitialise COM. Doing that with a
/// guard means an early return cannot leave the apartment initialised.
struct ComApartment {
    /// `false` when COM was already initialised on this thread by someone
    /// else, in which case we must not uninitialise it.
    owned: bool,
}

impl ComApartment {
    fn enter() -> Result<Self> {
        // SAFETY: called once per guard; the matching CoUninitialize is in Drop.
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) };
        if hr.is_ok() {
            return Ok(Self { owned: true });
        }
        // RPC_E_CHANGED_MODE: the thread is already in a different apartment.
        // S_FALSE: already initialised in the same mode; we did not take
        // ownership, so we must not uninitialise.
        if hr.0 == 1 {
            return Ok(Self { owned: false });
        }
        Err(Error::ScheduledTask {
            task: "<com>".into(),
            source_msg: format!("CoInitializeEx failed: {hr:?}"),
        })
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        if self.owned {
            // SAFETY: balances the CoInitializeEx in `enter`.
            unsafe { CoUninitialize() };
        }
    }
}

fn connect() -> Result<ITaskService> {
    // SAFETY: `TaskScheduler` is the documented CLSID for `ITaskService`.
    let service: ITaskService =
        unsafe { CoCreateInstance(&TaskScheduler as *const GUID, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| Error::ScheduledTask {
                task: "<task service>".into(),
                source_msg: format!("CoCreateInstance(TaskScheduler) failed: {e}"),
            })?;

    // Four empty VARIANTs means "connect to the local machine as me".
    let empty = VARIANT::default();
    // SAFETY: all four arguments are valid, initialised VARIANTs.
    unsafe { service.Connect(&empty, &empty, &empty, &empty) }.map_err(|e| {
        Error::ScheduledTask {
            task: "<task service>".into(),
            source_msg: format!("ITaskService::Connect failed: {e}"),
        }
    })?;

    Ok(service)
}

/// Enumerate every task in the scheduler tree.
///
/// `include_xml` fetches each task's definition, which is what lets the
/// scanner attribute a task to the product that created it — at the cost of
/// several hundred extra COM calls.
pub fn enumerate(include_xml: bool) -> Result<Vec<TaskInfo>> {
    let _com = ComApartment::enter()?;
    let service = connect()?;

    // SAFETY: the service is connected; `\` is the root folder path.
    let root =
        unsafe { service.GetFolder(&BSTR::from("\\")) }.map_err(|e| Error::ScheduledTask {
            task: "\\".into(),
            source_msg: format!("GetFolder(\\) failed: {e}"),
        })?;

    let mut out = Vec::new();
    let mut stack = vec![root];

    while let Some(folder) = stack.pop() {
        // --- tasks in this folder ---------------------------------------
        // SAFETY: `folder` is a live ITaskFolder.
        if let Ok(tasks) = unsafe { folder.GetTasks(0) } {
            // SAFETY: Count is a simple property read.
            let count = unsafe { tasks.Count() }.unwrap_or(0);
            // The collection is 1-based, which is a classic off-by-one here.
            for index in 1..=count {
                let variant = VARIANT::from(index);
                // SAFETY: `variant` holds a valid I4 index within range.
                let Ok(task) = (unsafe { tasks.get_Item(&variant) }) else {
                    continue;
                };
                // SAFETY: property reads on a live IRegisteredTask.
                unsafe {
                    let path = task.Path().map(|b| b.to_string()).unwrap_or_default();
                    let name = task.Name().map(|b| b.to_string()).unwrap_or_default();
                    let state = task.State().unwrap_or(TASK_STATE_DISABLED);
                    let enabled = task.Enabled().map(|b| b != VARIANT_FALSE).unwrap_or(false)
                        && state != TASK_STATE_DISABLED;
                    let xml = if include_xml {
                        task.Xml().ok().map(|b| b.to_string())
                    } else {
                        None
                    };
                    if !path.is_empty() {
                        out.push(TaskInfo {
                            path,
                            name,
                            enabled,
                            running: state == TASK_STATE_RUNNING,
                            xml,
                        });
                    }
                }
            }
        }

        // --- subfolders ---------------------------------------------------
        // SAFETY: `folder` is a live ITaskFolder.
        if let Ok(folders) = unsafe { folder.GetFolders(0) } {
            // SAFETY: Count is a simple property read.
            let count = unsafe { folders.Count() }.unwrap_or(0);
            for index in 1..=count {
                let variant = VARIANT::from(index);
                // SAFETY: `variant` holds a valid I4 index within range.
                if let Ok(sub) = unsafe { folders.get_Item(&variant) } {
                    stack.push(sub);
                }
            }
        }
    }

    out.sort_by_key(|t| t.path.to_lowercase());
    Ok(out)
}

/// Enable or disable one task by its full path.
///
/// A task that does not exist is reported as skipped, not as a failure:
/// Windows removes its own tasks between releases, and a stale rule should
/// not turn into a red error in the user's log.
pub fn set_enabled(path: &str, enabled: bool) -> Result<StepResult> {
    let _com = ComApartment::enter()?;
    let service = connect()?;

    let (folder_path, leaf) = split_task_path(path);

    // SAFETY: `service` is connected; the BSTR outlives the call.
    let Ok(folder) = (unsafe { service.GetFolder(&BSTR::from(folder_path.as_str())) }) else {
        return Ok(StepResult::skipped(format!(
            "scheduled-task folder `{folder_path}` does not exist"
        )));
    };

    // SAFETY: `folder` is live; the BSTR outlives the call.
    let Ok(task) = (unsafe { folder.GetTask(&BSTR::from(leaf.as_str())) }) else {
        return Ok(StepResult::skipped(format!(
            "scheduled task `{path}` is not registered"
        )));
    };

    // SAFETY: `task` is a live IRegisteredTask.
    unsafe { task.SetEnabled(if enabled { VARIANT_TRUE } else { VARIANT_FALSE }) }.map_err(
        |e| Error::ScheduledTask {
            task: path.to_string(),
            source_msg: format!("SetEnabled failed: {e}"),
        },
    )?;

    Ok(StepResult::ok(format!(
        "scheduled task `{path}` {}",
        if enabled { "enabled" } else { "disabled" }
    )))
}
