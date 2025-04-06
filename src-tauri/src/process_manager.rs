use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;

use windows::core::Free;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::{
    Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    },
    Threading::{OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE},
};

use crate::{error::DwallSettingsResult, DAEMON_EXE_PATH};

const INITIAL_BUFFER_SIZE: usize = 1024;
const BUFFER_INCREMENT: usize = 1024;

#[derive(Debug)]
struct HandleWrapper(HANDLE);

impl HandleWrapper {
    fn new(handle: HANDLE) -> Self {
        Self(handle)
    }
}

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        unsafe { self.0.free() };
        trace!("Released handle");
    }
}

pub fn find_daemon_process() -> DwallSettingsResult<Option<u32>> {
    // Get the daemon path
    let daemon_path = DAEMON_EXE_PATH.get().ok_or_else(|| {
        error!("Failed to retrieve daemon executable path");
        std::io::Error::new(std::io::ErrorKind::NotFound, "Daemon path not set")
    })?;

    // Get the daemon filename for initial filtering
    let daemon_filename = Path::new(daemon_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            error!("Invalid daemon path format");
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid daemon path format",
            )
        })?;

    // Normalize path for comparison
    let daemon_path_str = match daemon_path.to_str() {
        Some(path) => path,
        None => {
            error!("Daemon path contains invalid Unicode characters");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Daemon path contains invalid Unicode characters",
            )
            .into());
        }
    };

    // Create process snapshot
    let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
        Ok(snap) => {
            trace!("Created process snapshot successfully");
            snap
        }
        Err(e) => {
            error!(error = ?e, "Failed to create process snapshot");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create process snapshot: {}", e),
            )
            .into());
        }
    };

    let _snapshot_handle_wrapper = HandleWrapper::new(snapshot);

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

                // Get process executable name for initial filtering
                let exe_name = {
                    let name_ptr = process_entry.szExeFile.as_ptr();
                    let mut len = 0;
                    while len < process_entry.szExeFile.len() && *name_ptr.add(len) != 0 {
                        len += 1;
                    }
                    String::from_utf8_lossy(std::slice::from_raw_parts(name_ptr as *const u8, len))
                        .to_string()
                };

                // Initial filtering: check if executable name matches
                if !exe_name.eq_ignore_ascii_case(daemon_filename) {
                    if Process32Next(snapshot, &mut process_entry).is_err() {
                        break;
                    }
                    continue;
                }

                // Get full path of executable
                let mut filename_buffer = vec![0u16; INITIAL_BUFFER_SIZE];

                // Dynamically expand buffer until full path is obtained
                let filename_length = loop {
                    let filename_length = GetModuleFileNameExW(
                        Some(process_handle),
                        None,
                        filename_buffer.as_mut_slice(),
                    );

                    // Check if buffer needs expansion
                    if filename_length as usize >= filename_buffer.capacity() {
                        filename_buffer.resize(filename_buffer.capacity() + BUFFER_INCREMENT, 0);
                    } else {
                        break filename_length;
                    }
                };

                let _handle_wrapper = HandleWrapper::new(process_handle);

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

                        let is_match = is_daemon_process(full_path_str, daemon_path_str);

                        if is_match {
                            info!(
                                pid = process_entry.th32ProcessID,
                                path = full_path_str,
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

fn is_daemon_process(process_path: &str, expected_path: &str) -> bool {
    // Normalize paths for comparison
    let process_path = Path::new(process_path);
    let expected_path = Path::new(expected_path);

    // Try to get canonical paths
    match (process_path.canonicalize(), expected_path.canonicalize()) {
        (Ok(p1), Ok(p2)) => {
            // Compare using canonical paths
            match (p1.to_str(), p2.to_str()) {
                (Some(s1), Some(s2)) => s1.eq_ignore_ascii_case(s2),
                _ => false,
            }
        }
        // If canonicalization fails, compare using original paths
        _ => process_path.to_str().map_or(false, |p| {
            expected_path
                .to_str()
                .map_or(false, |e| p.eq_ignore_ascii_case(e))
        }),
    }
}

#[tauri::command]
pub fn kill_daemon() -> DwallSettingsResult<()> {
    // Find the daemon process
    match find_daemon_process()? {
        Some(pid) => {
            // Open process with termination rights
            let process_handle = unsafe {
                OpenProcess(PROCESS_TERMINATE, false, pid).map_err(|e| {
                    error!(error = ?e, pid = pid, "Failed to open process for termination");
                    std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!(
                            "Failed to open process with PID {} for termination: {}",
                            pid, e
                        ),
                    )
                })?
            };

            let _process_handle_wrapper = HandleWrapper::new(process_handle);

            // Terminate the process
            unsafe {
                TerminateProcess(process_handle, 0).map_err(|e| {
                    error!(error = ?e, pid = pid, "Failed to terminate daemon process");
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to terminate daemon process with PID {}: {}", pid, e),
                    )
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
