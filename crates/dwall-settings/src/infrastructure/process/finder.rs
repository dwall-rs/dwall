//! Process finder (pure technical)

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32, Process32First, Process32Next, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION};

use crate::error::{DwallSettingsError, DwallSettingsResult};

use super::HandleWrapper;
use super::path_comparison::is_same_process;

const INITIAL_BUFFER_SIZE: usize = 1024;
const BUFFER_INCREMENT: usize = 1024;

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

fn get_module_filename(process_handle: HANDLE) -> Result<OsString, DwallSettingsError> {
    let mut filename_buffer = vec![0u16; INITIAL_BUFFER_SIZE];

    let filename_length = unsafe {
        loop {
            let filename_length =
                GetModuleFileNameExW(Some(process_handle), None, filename_buffer.as_mut_slice());

            if filename_length as usize >= filename_buffer.capacity() {
                filename_buffer.resize(filename_buffer.capacity() + BUFFER_INCREMENT, 0);
            } else {
                break filename_length;
            }
        }
    };

    if filename_length > 0 {
        filename_buffer.truncate(filename_length as usize);
        Ok(OsString::from_wide(&filename_buffer))
    } else {
        Err(DwallSettingsError::Other(
            "Failed to get module filename".to_string(),
        ))
    }
}

fn check_process_path(pid: u32, expected_path: &str) -> DwallSettingsResult<Option<u32>> {
    let process_handle = unsafe {
        match OpenProcess(PROCESS_QUERY_INFORMATION, false, pid) {
            Ok(handle) => HandleWrapper::new(handle),
            Err(_) => return Ok(None),
        }
    };

    let full_path = match get_module_filename(process_handle.as_raw()) {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };

    if let Some(full_path_str) = full_path.to_str()
        && is_same_process(full_path_str, expected_path)
    {
        return Ok(Some(pid));
    }

    Ok(None)
}

/// Find a process by comparing executable paths
pub fn find_process_by_path(target_path: &Path) -> DwallSettingsResult<Option<u32>> {
    let target_filename = target_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            DwallSettingsError::Other("Could not extract filename from path".to_string())
        })?;

    let target_path_str = target_path
        .to_str()
        .ok_or_else(|| DwallSettingsError::Other("Path contains invalid Unicode".to_string()))?;

    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(DwallSettingsError::Windows)?;

    let snapshot = HandleWrapper::new(snapshot);

    let mut process_entry = PROCESSENTRY32 {
        dwSize: std::mem::size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };

    unsafe {
        if Process32First(snapshot.as_raw(), &mut process_entry).is_ok() {
            loop {
                let exe_name = get_process_exe_name(&process_entry);

                if exe_name.eq_ignore_ascii_case(target_filename)
                    && let Some(pid) =
                        check_process_path(process_entry.th32ProcessID, target_path_str)?
                {
                    return Ok(Some(pid));
                }

                if Process32Next(snapshot.as_raw(), &mut process_entry).is_err() {
                    break;
                }
            }
        }
    }

    Ok(None)
}
