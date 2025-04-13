use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;

use thiserror::Error;
use windows::core::{Error as WindowsError, Free};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::{
    Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
    },
    Threading::{OpenProcess, TerminateProcess, PROCESS_QUERY_INFORMATION, PROCESS_TERMINATE},
};

use crate::{
    error::{DwallSettingsError, DwallSettingsResult},
    DAEMON_EXE_PATH,
};

const INITIAL_BUFFER_SIZE: usize = 1024;
const BUFFER_INCREMENT: usize = 1024;

/// Custom error type for process management operations
#[derive(Debug, Error)]
pub enum ProcessManagerError {
    #[error("Failed to retrieve daemon executable path: {0}")]
    DaemonPathNotFound(String),

    #[error("Invalid daemon path format: {0}")]
    InvalidDaemonPath(String),

    #[error("Path contains invalid Unicode characters: {0}")]
    InvalidUnicode(String),

    #[error("Failed to create process snapshot: {0}")]
    SnapshotCreationFailed(#[from] WindowsError),

    #[error("Failed to open process: {0}")]
    ProcessOpenFailed(WindowsError),

    #[error("Failed to terminate process with PID {pid}: {error}")]
    ProcessTerminationFailed { pid: u32, error: WindowsError },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<ProcessManagerError> for DwallSettingsError {
    fn from(error: ProcessManagerError) -> Self {
        match error {
            ProcessManagerError::Io(e) => DwallSettingsError::Io(e),
            ProcessManagerError::SnapshotCreationFailed(e) => DwallSettingsError::Windows(e),
            _ => DwallSettingsError::Daemon(error.to_string()),
        }
    }
}

/// Safe wrapper for Windows HANDLE that automatically closes the handle when dropped
#[derive(Debug)]
struct HandleWrapper(HANDLE);

impl HandleWrapper {
    /// Creates a new HandleWrapper from a Windows HANDLE
    fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

    /// Returns the raw HANDLE for use with Windows APIs
    fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for HandleWrapper {
    fn drop(&mut self) {
        // Only log and close if the handle is valid
        if !self.0.is_invalid() {
            let c_void_addr = self.0 .0.addr();
            unsafe { self.0.free() };
            trace!(addr = c_void_addr, "Released handle");
        }
    }
}

/// Helper function to extract daemon filename from path
fn get_daemon_filename(daemon_path: &Path) -> Result<&str, ProcessManagerError> {
    daemon_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            error!("Invalid daemon path format");
            ProcessManagerError::InvalidDaemonPath("Could not extract filename".to_string())
        })
}

/// Helper function to get string representation of path
fn path_to_string(path: &Path) -> Result<&str, ProcessManagerError> {
    path.to_str().ok_or_else(|| {
        error!("Path contains invalid Unicode characters");
        ProcessManagerError::InvalidUnicode("Path contains invalid Unicode characters".to_string())
    })
}

/// Helper function to safely get process executable name
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

/// Helper function to get module filename with dynamic buffer resizing
fn get_module_filename(process_handle: HANDLE) -> Result<OsString, ProcessManagerError> {
    let mut filename_buffer = vec![0u16; INITIAL_BUFFER_SIZE];

    // Dynamically expand buffer until full path is obtained
    let filename_length = unsafe {
        loop {
            let filename_length =
                GetModuleFileNameExW(Some(process_handle), None, filename_buffer.as_mut_slice());

            // Check if buffer needs expansion
            if filename_length as usize >= filename_buffer.capacity() {
                filename_buffer.resize(filename_buffer.capacity() + BUFFER_INCREMENT, 0);
            } else {
                break filename_length;
            }
        }
    };

    if filename_length > 0 {
        // Truncate the buffer to actual length and convert to OsString
        filename_buffer.truncate(filename_length as usize);
        Ok(OsString::from_wide(&filename_buffer))
    } else {
        Err(ProcessManagerError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get module filename",
        )))
    }
}

/// Finds the daemon process by comparing executable paths
pub fn find_daemon_process() -> DwallSettingsResult<Option<u32>> {
    // Get the daemon path
    let daemon_path = DAEMON_EXE_PATH.get().ok_or_else(|| {
        error!("Failed to retrieve daemon executable path");
        ProcessManagerError::DaemonPathNotFound("Daemon path not set".to_string())
    })?;

    // Get the daemon filename for initial filtering
    let daemon_filename = get_daemon_filename(daemon_path)?;

    // Normalize path for comparison
    let daemon_path_str = path_to_string(daemon_path)?;

    // Create process snapshot
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.map_err(|e| {
        error!(error = ?e, "Failed to create process snapshot");
        ProcessManagerError::SnapshotCreationFailed(e)
    })?;

    let snapshot = HandleWrapper::new(snapshot);

    let mut process_entry = PROCESSENTRY32 {
        dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    let result = unsafe {
        // Start iterating through processes
        if Process32First(snapshot.as_raw(), &mut process_entry).is_ok() {
            find_matching_process(
                &mut process_entry,
                snapshot.as_raw(),
                daemon_filename,
                daemon_path_str,
            )
        } else {
            error!("Failed to get first process in snapshot");
            Ok(None)
        }
    };

    result
}

/// Helper function to find matching process by iterating through processes
unsafe fn find_matching_process(
    process_entry: &mut PROCESSENTRY32,
    snapshot: HANDLE,
    daemon_filename: &str,
    daemon_path_str: &str,
) -> DwallSettingsResult<Option<u32>> {
    loop {
        // Get process executable name for initial filtering
        let exe_name = get_process_exe_name(process_entry);

        // Initial filtering: check if executable name matches
        if exe_name.eq_ignore_ascii_case(daemon_filename) {
            // Try to get the full path of the executable
            if let Some(pid) = check_process_path(process_entry.th32ProcessID, daemon_path_str)? {
                return Ok(Some(pid));
            }
        }

        // Move to next process
        if Process32Next(snapshot, process_entry).is_err() {
            break;
        }
    }

    warn!(
        path = daemon_path_str,
        "No matching daemon process found for path"
    );
    Ok(None)
}

/// Helper function to check if a process matches the expected daemon path
fn check_process_path(pid: u32, expected_path: &str) -> DwallSettingsResult<Option<u32>> {
    // Try to get the full path of the executable
    let process_handle = unsafe {
        OpenProcess(PROCESS_QUERY_INFORMATION, false, pid).map_err(|e| {
            trace!(pid = pid, error = ?e, "Could not open process");
            ProcessManagerError::ProcessOpenFailed(e)
        })
    };

    // Skip processes we can't open
    let process_handle = match process_handle {
        Ok(handle) => HandleWrapper::new(handle),
        Err(_) => return Ok(None),
    };

    // Get full path of executable
    let full_path = match get_module_filename(process_handle.as_raw()) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };

    // Compare paths
    if let Some(full_path_str) = full_path.to_str() {
        trace!(pid = pid, path = full_path_str, "Checking process");

        if is_daemon_process(full_path_str, expected_path) {
            info!(
                pid = pid,
                path = full_path_str,
                "Found matching daemon process"
            );
            return Ok(Some(pid));
        }
    }

    Ok(None)
}

/// Compares two paths to determine if they refer to the same file
fn is_daemon_process(process_path: &str, expected_path: &str) -> bool {
    let process_path = Path::new(process_path);
    let expected_path = Path::new(expected_path);

    // In debug mode, only compare filenames as we can't match daemon process paths in production environment
    if cfg!(debug_assertions) {
        return process_path.file_name() == expected_path.file_name();
    }

    // First try to compare canonical paths (resolves symlinks, etc.)
    if let (Ok(p1), Ok(p2)) = (process_path.canonicalize(), expected_path.canonicalize()) {
        #[cfg(windows)]
        {
            // Case-insensitive comparison on Windows platform
            if let (Some(s1), Some(s2)) = (p1.to_str(), p2.to_str()) {
                return s1.eq_ignore_ascii_case(s2);
            }
        }

        #[cfg(unix)]
        {
            // Case-sensitive comparison on Unix platform
            return p1 == p2;
        }
    }

    // Fallback to comparing original paths if canonicalization fails
    #[cfg(windows)]
    {
        process_path.to_str().is_some_and(|p| {
            expected_path
                .to_str()
                .is_some_and(|e| p.eq_ignore_ascii_case(e))
        })
    }

    #[cfg(unix)]
    {
        process_path == expected_path
    }
}

/// Terminates the daemon process if it's running
#[tauri::command]
pub fn kill_daemon() -> DwallSettingsResult<()> {
    // Find the daemon process
    match find_daemon_process()? {
        Some(pid) => {
            // Open process with termination rights
            let process_handle = unsafe {
                OpenProcess(PROCESS_TERMINATE, false, pid).map_err(|e| {
                    error!(error = ?e, pid = pid, "Failed to open process for termination");
                    ProcessManagerError::ProcessTerminationFailed { pid, error: e }
                })
            }?;

            let process_handle = HandleWrapper::new(process_handle);

            // Terminate the process
            unsafe {
                TerminateProcess(process_handle.as_raw(), 0).map_err(|e| {
                    error!(error = ?e, pid = pid, "Failed to terminate daemon process");
                    ProcessManagerError::ProcessTerminationFailed { pid, error: e }
                })
            }?;

            info!(pid = pid, "Successfully terminated daemon process with PID");
            Ok(())
        }
        None => {
            warn!("No daemon process found to terminate");
            Ok(())
        }
    }
}
