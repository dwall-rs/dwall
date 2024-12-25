use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::{
    Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    },
    Threading::{OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE},
};

use crate::{error::DwallSettingsResult, DAEMON_EXE_PATH};

pub fn find_daemon_process() -> DwallSettingsResult<Option<u32>> {
    // Get the daemon path
    let daemon_path = DAEMON_EXE_PATH.get().ok_or_else(|| {
        error!("Failed to retrieve daemon executable path");
        std::io::Error::new(std::io::ErrorKind::NotFound, "Daemon path not set")
    })?;

    // Convert path to string for comparison
    let daemon_path_str = match daemon_path.to_str() {
        Some(path) => path,
        None => {
            error!("Invalid daemon path encoding");
            return Ok(None);
        }
    };

    // Create a snapshot of current processes
    let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
        Ok(snap) => {
            trace!("Created process snapshot successfully");
            snap
        }
        Err(e) => {
            error!(error = ?e, "Failed to create process snapshot");
            return Ok(None);
        }
    };

    let mut process_entry = PROCESSENTRY32 {
        dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    unsafe {
        // Start iterating through processes
        if Process32First(snapshot, &mut process_entry).is_ok() {
            loop {
                // Try to get the full path of the executable
                let process_handle = match OpenProcess(
                    PROCESS_TERMINATE | PROCESS_QUERY_INFORMATION,
                    false,
                    process_entry.th32ProcessID,
                ) {
                    Ok(handle) => handle,
                    Err(e) => {
                        debug!(
                            pid = process_entry.th32ProcessID,
                            error = ?e,
                            "Could not open process"
                        );
                        if Process32Next(snapshot, &mut process_entry).is_err() {
                            break;
                        }
                        continue;
                    }
                };

                // Get the full path of the executable
                let mut filename_buffer = vec![0u16; 1024];
                let filename_length =
                    GetModuleFileNameExW(process_handle, None, filename_buffer.as_mut_slice());

                if filename_length > 0 {
                    // Truncate the buffer to actual length and convert to OsString
                    filename_buffer.truncate(filename_length as usize);
                    let full_path = OsString::from_wide(&filename_buffer);

                    // Compare paths
                    if let Some(full_path_str) = full_path.to_str() {
                        trace!(
                            pid = process_entry.th32ProcessID,
                            path = full_path_str,
                            "Checking process",
                        );

                        if cfg!(debug_assertions) {
                            use std::ffi::OsStr;
                            use std::path::Path;
                            let full_path = Path::new(full_path_str);
                            if full_path.file_name() == Some(OsStr::new("dwall.exe")) {
                                info!(
                                    pid = process_entry.th32ProcessID,
                                    "Found matching daemon process",
                                );
                                return Ok(Some(process_entry.th32ProcessID));
                            }
                        } else if full_path_str.eq_ignore_ascii_case(daemon_path_str) {
                            info!(
                                pid = process_entry.th32ProcessID,
                                "Found matching daemon process",
                            );
                            return Ok(Some(process_entry.th32ProcessID));
                        }
                    }
                }

                // Move to next process
                if Process32Next(snapshot, &mut process_entry).is_err() {
                    break;
                }
            }

            warn!(
                path = daemon_path_str,
                "No matching daemon process found for path",
            );
        } else {
            error!("Failed to get first process in snapshot");
        }
    }

    Ok(None)
}

#[tauri::command]
pub fn kill_daemon() -> DwallSettingsResult<()> {
    // Find the daemon process
    match find_daemon_process()? {
        Some(pid) => {
            // Open process with termination rights
            let process_handle = unsafe {
                OpenProcess(PROCESS_TERMINATE, false, pid).map_err(|e| {
                    error!(error = ?e, "Failed to open process for termination");
                    e
                })?
            };

            // Terminate the process
            unsafe {
                TerminateProcess(process_handle, 0).map_err(|e| {
                    error!(error = ?e, "Failed to terminate daemon process");
                    e
                })?
            };

            info!(pid = pid, "Successfully terminated daemon process with PID");
            Ok(())
        }
        None => {
            warn!("No daemon process found to terminate");
            Ok(())
        }
    }
}
