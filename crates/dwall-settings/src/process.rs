//! Windows process management: find and terminate the daemon.

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};
use windows::core::Free;

use crate::error::{DwallSettingsError, DwallSettingsResult};

/// Executable name of the daemon, as built in `crates/dwall`.
const DAEMON_EXE_NAME: &str = "dwall.exe";

/// Safe wrapper for a Windows HANDLE
#[derive(Debug)]
struct HandleWrapper(HANDLE);

impl HandleWrapper {
    fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe { self.0.free() };
        }
    }
}

fn get_process_exe_name(process_entry: &PROCESSENTRY32) -> String {
    unsafe {
        let name_ptr = process_entry.szExeFile.as_ptr();
        let mut len = 0;
        while len < process_entry.szExeFile.len() && *name_ptr.add(len) != 0 {
            len += 1;
        }
        String::from_utf8_lossy(std::slice::from_raw_parts(name_ptr as *const u8, len)).to_string()
    }
}

/// Finds the running daemon process, wherever it was installed.
///
/// Matching on the executable name — rather than on the path the settings app
/// expects next to itself — is what makes a portable or headless `dwall.exe`
/// visible: the installer's copy, one in `C:\tools` and a debug build are all the
/// same product, and the window has to be able to see and stop whichever one
/// actually runs. For a plain "is it running?" check prefer
/// [`crate::daemon::is_daemon_running`], which reads the daemon's mutex instead of
/// walking the process list.
pub fn find_daemon_process() -> DwallSettingsResult<Option<u32>> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(DwallSettingsError::Windows)?;

    let snapshot = HandleWrapper::new(snapshot);
    let own_pid = std::process::id();

    let mut process_entry = PROCESSENTRY32 {
        dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    unsafe {
        if Process32First(snapshot.as_raw(), &mut process_entry).is_ok() {
            loop {
                if process_entry.th32ProcessID != own_pid
                    && get_process_exe_name(&process_entry).eq_ignore_ascii_case(DAEMON_EXE_NAME)
                {
                    return Ok(Some(process_entry.th32ProcessID));
                }

                if Process32Next(snapshot.as_raw(), &mut process_entry).is_err() {
                    break;
                }
            }
        }
    }

    Ok(None)
}

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
