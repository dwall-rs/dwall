use std::mem;

use windows::core::Error as WindowsError;
use windows::Win32::{
    Devices::Display::{
        DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME,
        QDC_ALL_PATHS,
    },
    Foundation::ERROR_SUCCESS,
};

use crate::{monitor::error::MonitorError, DwallResult};

/// Flag indicating that a display path is active
const DISPLAYCONFIG_PATH_ACTIVE: u32 = 0x00000001;

/// Represents a display path with adapter and target identifiers
///
/// This structure is used to identify a specific display device
/// in the Windows display configuration system.
pub struct DisplayPath {
    pub adapter_id: u32,
    pub target_id: u32,
}

/// Queries all active display paths in the system
///
/// This function retrieves information about all active display paths
/// by using the Windows Display Configuration API.
///
/// # Returns
/// - `Ok(Vec<DisplayPath>)` - A list of active display paths
/// - `Err(DwallError)` - If any Windows API call fails
pub fn query_display_paths() -> DwallResult<Vec<DisplayPath>> {
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
///
/// This function retrieves detailed information about a display device
/// using its adapter and target identifiers.
///
/// # Arguments
/// * `adapter_id` - The adapter identifier
/// * `target_id` - The target identifier
///
/// # Returns
/// - `Ok(DISPLAYCONFIG_TARGET_DEVICE_NAME)` - The target device information
/// - `Err(DwallError)` - If the Windows API call fails
pub fn query_target_name(
    adapter_id: u32,
    target_id: u32,
) -> DwallResult<DISPLAYCONFIG_TARGET_DEVICE_NAME> {
    debug!(adapter_id, target_id, "Querying target device name");

    // Initialize the target name structure with zeros
    let mut target_name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { mem::zeroed() };

    // Set up the header with the required information
    target_name.header = DISPLAYCONFIG_DEVICE_INFO_HEADER {
        size: mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
        adapterId: windows::Win32::Foundation::LUID {
            LowPart: adapter_id,
            HighPart: 0,
        },
        id: target_id,
        r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    };

    // Call the Windows API to get the target device name
    let result = unsafe { DisplayConfigGetDeviceInfo(&mut target_name as *mut _ as *mut _) };
    if result != 0 {
        let win_error = WindowsError::from_win32();
        error!(error = ?win_error, "Failed to get target name, error code: {}", result);
        return Err(MonitorError::GetTargetName(win_error).into());
    }

    debug!("Got target name");
    Ok(target_name)
}

/// Gets the required buffer sizes for display configuration queries
///
/// This function determines how much memory is needed to store
/// the display configuration information.
///
/// # Returns
/// - `Ok((path_count, mode_count))` - The number of paths and modes
/// - `Err(DwallError)` - If the Windows API call fails
fn get_buffer_sizes() -> DwallResult<(u32, u32)> {
    let mut path_count = 0;
    let mut mode_count = 0;

    // Call the Windows API to get the required buffer sizes
    let error =
        unsafe { GetDisplayConfigBufferSizes(QDC_ALL_PATHS, &mut path_count, &mut mode_count) };

    if error != ERROR_SUCCESS {
        error!(error = ?error, "Failed to get buffer sizes");
        return Err(MonitorError::GetBufferSizes(error).into());
    }
    debug!(path_count, mode_count, "Got buffer sizes");

    Ok((path_count, mode_count))
}

/// Queries the display configuration
///
/// This function retrieves detailed information about all display paths
/// and modes in the system.
///
/// # Arguments
/// * `path_count` - Mutable reference to the number of paths
/// * `mode_count` - Mutable reference to the number of modes
///
/// # Returns
/// - `Ok((paths, modes))` - The display paths and modes information
/// - `Err(DwallError)` - If the Windows API call fails
fn query_config(
    path_count: &mut u32,
    mode_count: &mut u32,
) -> DwallResult<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    // Allocate vectors with the required sizes
    let mut paths = vec![Default::default(); *path_count as usize];
    let mut modes = vec![Default::default(); *mode_count as usize];

    // Call the Windows API to query the display configuration
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
        return Err(MonitorError::QueryDisplayConfig(err).into());
    }

    // Truncate the vectors to the actual number of items returned
    paths.truncate(*path_count as usize);
    modes.truncate(*mode_count as usize);
    debug!(path_count, "Got display count");

    Ok((paths, modes))
}
