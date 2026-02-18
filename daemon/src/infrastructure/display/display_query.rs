//! Display configuration query utilities for Windows

use std::mem;

use windows::Win32::{
    Devices::Display::{
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME,
        DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QDC_ALL_PATHS, QueryDisplayConfig,
    },
    Foundation::ERROR_SUCCESS,
};
use windows::core::Error as WindowsError;

use crate::DwallResult;

use super::error::DisplayError;

const DISPLAYCONFIG_PATH_ACTIVE: u32 = 0x00000001;

/// Represents a display path with adapter and target identifiers
pub(crate) struct DisplayPath {
    pub(crate) adapter_id: u32,
    pub(crate) target_id: u32,
}

/// Queries all active display paths in the system
pub(crate) fn query_display_paths() -> DwallResult<Vec<DisplayPath>> {
    let (mut path_count, mut mode_count) = get_buffer_sizes()?;
    let (paths, _) = query_config(&mut path_count, &mut mode_count)?;

    Ok(paths
        .into_iter()
        .filter(|path| path.flags & DISPLAYCONFIG_PATH_ACTIVE != 0)
        .map(|path| DisplayPath {
            adapter_id: path.targetInfo.adapterId.LowPart,
            target_id: path.targetInfo.id,
        })
        .collect())
}

/// Queries the target device name for a specific display adapter and target
pub(crate) fn query_target_name(
    adapter_id: u32,
    target_id: u32,
) -> DwallResult<DISPLAYCONFIG_TARGET_DEVICE_NAME> {
    debug!(adapter_id, target_id, "Querying target device name");

    let mut target_name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { mem::zeroed() };

    target_name.header = DISPLAYCONFIG_DEVICE_INFO_HEADER {
        size: mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
        adapterId: windows::Win32::Foundation::LUID {
            LowPart: adapter_id,
            HighPart: 0,
        },
        id: target_id,
        r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    };

    let result = unsafe { DisplayConfigGetDeviceInfo(&mut target_name as *mut _ as *mut _) };
    if result != 0 {
        let win_error = WindowsError::from_win32();
        error!(error = ?win_error, "Failed to get target name, error code: {}", result);
        return Err(DisplayError::GetTargetName(win_error).into());
    }

    debug!("Got target name");
    Ok(target_name)
}

fn get_buffer_sizes() -> DwallResult<(u32, u32)> {
    let mut path_count = 0;
    let mut mode_count = 0;

    let error =
        unsafe { GetDisplayConfigBufferSizes(QDC_ALL_PATHS, &mut path_count, &mut mode_count) };

    if error != ERROR_SUCCESS {
        error!(error = ?error, "Failed to get buffer sizes");
        return Err(DisplayError::GetBufferSizes(error).into());
    }
    debug!(path_count, mode_count, "Got buffer sizes");

    Ok((path_count, mode_count))
}

fn query_config(
    path_count: &mut u32,
    mode_count: &mut u32,
) -> DwallResult<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    let mut paths = vec![Default::default(); *path_count as usize];
    let mut modes = vec![Default::default(); *mode_count as usize];

    let err = unsafe {
        QueryDisplayConfig(
            QDC_ALL_PATHS,
            path_count,
            paths.as_mut_ptr(),
            mode_count,
            modes.as_mut_ptr(),
            None,
        )
    };

    if err != ERROR_SUCCESS {
        error!(error = ?err, "Failed to get display config");
        return Err(DisplayError::QueryDisplayConfig(err).into());
    }

    paths.truncate(*path_count as usize);
    modes.truncate(*mode_count as usize);
    debug!(path_count, "Got display count");

    Ok((paths, modes))
}
