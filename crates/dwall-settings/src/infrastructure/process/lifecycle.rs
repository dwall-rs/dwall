//! Process lifecycle management (pure technical)

use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};

use crate::error::{DwallSettingsError, DwallSettingsResult};

use super::HandleWrapper;

/// Terminate a process by PID
pub fn terminate_process(pid: u32) -> DwallSettingsResult<()> {
    let process_handle =
        unsafe { OpenProcess(PROCESS_TERMINATE, false, pid).map_err(DwallSettingsError::Windows)? };

    let process_handle = HandleWrapper::new(process_handle);

    unsafe {
        TerminateProcess(process_handle.as_raw(), 0).map_err(DwallSettingsError::Windows)?;
    }

    Ok(())
}
