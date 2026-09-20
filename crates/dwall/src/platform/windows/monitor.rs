//! Windows monitor enumeration.
//!
//! Queries the display configuration (active paths + target names) and maps
//! each monitor device path to a friendly name via the device installer API.

use std::{
    cell::RefCell,
    collections::HashMap,
    mem,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use windows::Win32::{
    Devices::DeviceAndDriverInstallation::{
        DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, HDEVINFO, SP_DEVICE_INTERFACE_DATA,
        SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA, SPDRP_FRIENDLYNAME,
        SetupDiEnumDeviceInfo, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
        SetupDiGetDeviceInterfaceDetailW, SetupDiGetDeviceRegistryPropertyW,
    },
    Devices::Display::{
        DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_DEVICE_INFO_HEADER,
        DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_TARGET_DEVICE_NAME,
        DisplayConfigGetDeviceInfo, GUID_DEVINTERFACE_MONITOR, GetDisplayConfigBufferSizes,
        QDC_ALL_PATHS, QueryDisplayConfig,
    },
    Foundation::ERROR_SUCCESS,
};
use windows::core::{Error as WindowsError, Free, GUID, PCWSTR};

use crate::DwallResult;
use crate::monitor::MonitorProvider;
use crate::utils::WideStringRead;

// ── Errors ──────────────────────────────────────────────────────────────────

/// Display operation related errors
#[derive(Debug, thiserror::Error)]
pub enum DisplayError {
    #[error("Unable to get monitor device collection: {0:?}")]
    GetDeviceInfoSet(#[source] Option<WindowsError>),

    #[error("Unable to get monitor device information: {0:?}")]
    GetDeviceInfo(#[source] WindowsError),

    #[error("Unable to get target device name: {0:?}")]
    GetTargetName(#[source] WindowsError),

    #[error("Unable to find matching device")]
    MatchDevice,

    #[error("Unable to get device friendly name: {0:?}")]
    GetFriendlyName(#[source] WindowsError),

    #[error("Unable to get buffer sizes: {0:?}")]
    GetBufferSizes(windows::Win32::Foundation::WIN32_ERROR),

    #[error("Failed to query display configuration: {0:?}")]
    QueryDisplayConfig(windows::Win32::Foundation::WIN32_ERROR),

    #[error("Failed to get device registry property: {0:?}")]
    GetDeviceRegistryProperty(#[source] WindowsError),
}

// ── Display configuration queries ───────────────────────────────────────────

const DISPLAYCONFIG_PATH_ACTIVE: u32 = 0x00000001;

/// Represents a display path with adapter and target identifiers
struct DisplayPath {
    adapter_id: u32,
    target_id: u32,
}

/// Queries all active display paths in the system
fn query_display_paths() -> DwallResult<Vec<DisplayPath>> {
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
fn query_target_name(
    adapter_id: u32,
    target_id: u32,
) -> DwallResult<DISPLAYCONFIG_TARGET_DEVICE_NAME> {
    debug!(
        adapter_id = adapter_id,
        target_id = target_id,
        "Querying target device name"
    );

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
    debug!(
        path_count = path_count,
        mode_count = mode_count,
        "Got buffer sizes"
    );

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
    debug!(path_count = path_count, "Got display count");

    Ok((paths, modes))
}

// ── Device installer queries ────────────────────────────────────────────────

/// RAII wrapper for HDEVINFO to ensure proper resource cleanup
struct DeviceInfoSet(HDEVINFO);

impl DeviceInfoSet {
    fn new(device_guid: &GUID) -> DwallResult<Self> {
        debug!(guid = ?device_guid, "Creating new DeviceInfoSet");
        let device_info_set = unsafe {
            SetupDiGetClassDevsW(
                Some(device_guid),
                None,
                None,
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            )
        }
        .map_err(|e| {
            error!(error = %e, "Failed to get device info set");
            DisplayError::GetDeviceInfoSet(Some(e))
        })?;

        if device_info_set.is_invalid() {
            error!("Device info set is invalid");
            return Err(DisplayError::GetDeviceInfoSet(None).into());
        }

        debug!("DeviceInfoSet created successfully");
        Ok(Self(device_info_set))
    }

    fn as_raw(&self) -> HDEVINFO {
        self.0
    }
}

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        debug!("Freeing DeviceInfoSet");
        unsafe { self.0.free() };
    }
}

/// Query the friendly name of a device by its device path and GUID
fn query_device_friendly_name(device_path: &str, device_guid: &GUID) -> DwallResult<String> {
    debug!(
        device_path = device_path,
        guid = ?device_guid,
        "Querying device friendly name"
    );

    let device_info_set = DeviceInfoSet::new(device_guid)?;
    let device_index = find_device_by_path(&device_info_set, device_guid, device_path)?;
    debug!(device_index = device_index, "Found device at index");

    let device_info_data = get_device_info_data(&device_info_set, device_index)?;
    let friendly_name = get_device_friendly_name(&device_info_set, &device_info_data)?;

    debug!(
        device_path = device_path,
        friendly_name = friendly_name,
        "Successfully retrieved device friendly name"
    );

    Ok(friendly_name)
}

fn find_device_by_path(
    device_info_set: &DeviceInfoSet,
    device_guid: &GUID,
    target_path: &str,
) -> DwallResult<u32> {
    debug!(
        target_path = target_path,
        guid = ?device_guid,
        "Finding device by path"
    );

    let mut device_interface_data = SP_DEVICE_INTERFACE_DATA {
        cbSize: mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
        InterfaceClassGuid: GUID::zeroed(),
        Flags: 0,
        Reserved: 0,
    };

    let mut index = 0;
    let device_info_handle = device_info_set.as_raw();

    while unsafe {
        SetupDiEnumDeviceInterfaces(
            device_info_handle,
            None,
            device_guid,
            index,
            &mut device_interface_data,
        )
        .is_ok()
    } {
        if let Some(current_path) = get_device_path(device_info_set, &device_interface_data)? {
            debug!(
                index = index,
                current_path = current_path,
                "Checking device path"
            );

            if current_path.eq_ignore_ascii_case(target_path) {
                debug!(index = index, "Found matching device");
                return Ok(index);
            }
        }

        index += 1;
    }

    error!(
        target_path = target_path,
        "Failed to find device matching path"
    );
    Err(DisplayError::MatchDevice.into())
}

fn get_device_path(
    device_info_set: &DeviceInfoSet,
    device_interface_data: &SP_DEVICE_INTERFACE_DATA,
) -> DwallResult<Option<String>> {
    unsafe {
        let mut required_size = 0;
        let _ = SetupDiGetDeviceInterfaceDetailW(
            device_info_set.as_raw(),
            device_interface_data,
            None,
            0,
            Some(&mut required_size),
            None,
        );

        let detail_data_size = mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
        let buffer_size = required_size.max(detail_data_size);

        let mut buffer = vec![0u8; buffer_size as usize];
        let detail_data = buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
        (*detail_data).cbSize = detail_data_size;

        if let Err(e) = SetupDiGetDeviceInterfaceDetailW(
            device_info_set.as_raw(),
            device_interface_data,
            Some(detail_data),
            buffer_size,
            None,
            None,
        ) {
            error!(error = %e, "Failed to get device interface detail");
            return Ok(None);
        }

        let device_path = PCWSTR((*detail_data).DevicePath.as_ptr());
        match device_path.to_string() {
            Ok(path) => Ok(Some(path)),
            Err(e) => {
                error!(error = %e, "Failed to convert device path to string");
                Ok(None)
            }
        }
    }
}

fn get_device_info_data(
    device_info_set: &DeviceInfoSet,
    device_index: u32,
) -> DwallResult<SP_DEVINFO_DATA> {
    debug!(device_index = device_index, "Getting device info data");

    let mut device_info_data = SP_DEVINFO_DATA {
        cbSize: mem::size_of::<SP_DEVINFO_DATA>() as u32,
        ClassGuid: GUID::zeroed(),
        DevInst: 0,
        Reserved: 0,
    };

    unsafe {
        SetupDiEnumDeviceInfo(
            device_info_set.as_raw(),
            device_index,
            &mut device_info_data,
        )
    }
    .map_err(|e| {
        error!(
            device_index = device_index,
            error = %e,
            "Failed to get device info"
        );
        DisplayError::GetDeviceInfo(e)
    })?;

    Ok(device_info_data)
}

fn get_device_friendly_name(
    device_info_set: &DeviceInfoSet,
    device_info_data: &SP_DEVINFO_DATA,
) -> DwallResult<String> {
    debug!("Getting device friendly name");

    let mut required_size = 0;
    let mut buffer = vec![0u8; 512];

    unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            device_info_set.as_raw(),
            device_info_data,
            SPDRP_FRIENDLYNAME,
            None,
            Some(buffer.as_mut()),
            Some(&mut required_size),
        )
        .map_err(|e| {
            error!(
                error = %e,
                "Failed to get device registry property"
            );
            DisplayError::GetDeviceRegistryProperty(e)
        })?;
    }

    let u16_data: Vec<u16> = buffer
        .as_chunks::<2>()
        .0
        .iter()
        .take((required_size / 2) as usize)
        .map(|chunk| u16::from_le_bytes(*chunk))
        .collect();

    Ok(u16_data.to_string())
}

// ── Monitor provider ────────────────────────────────────────────────────────

/// Cache expiration time: reduces API calls by 90% at negligible memory cost.
const CACHE_EXPIRY_SECONDS: u64 = 300; // 5 minutes

/// Display monitor information with serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayMonitor {
    device_path: String,
    friendly_name: String,
    position_index: Option<u32>,
}

impl DisplayMonitor {
    fn new(device_path: String, friendly_name: String, position_index: Option<u32>) -> Self {
        Self {
            device_path,
            friendly_name,
            position_index,
        }
    }

    /// Test-only constructor for creating DisplayMonitor instances in tests
    pub fn new_test(
        device_path: String,
        friendly_name: String,
        position_index: Option<u32>,
    ) -> Self {
        Self::new(device_path, friendly_name, position_index)
    }

    pub fn device_path(&self) -> &str {
        &self.device_path
    }

    pub fn friendly_name(&self) -> &str {
        &self.friendly_name
    }

    pub fn position_index(&self) -> Option<u32> {
        self.position_index
    }
}

/// Cache structure for monitor information with optimized expiration
struct MonitorInfoCache {
    data: HashMap<String, DisplayMonitor>,
    updated_at: Instant,
    expiry_duration: Duration,
}

impl MonitorInfoCache {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            updated_at: Instant::now(),
            expiry_duration: Duration::from_secs(CACHE_EXPIRY_SECONDS),
        }
    }

    fn is_valid(&self) -> bool {
        !self.data.is_empty() && self.updated_at.elapsed() <= self.expiry_duration
    }

    fn update(&mut self, monitors: HashMap<String, DisplayMonitor>) {
        self.data = monitors;
        self.updated_at = Instant::now();
    }
}

/// Provider for display monitor information with optimized caching
pub struct DisplayMonitorProvider {
    cache: RefCell<MonitorInfoCache>,
}

impl Default for DisplayMonitorProvider {
    fn default() -> Self {
        Self {
            cache: RefCell::new(MonitorInfoCache::new()),
        }
    }
}

impl DisplayMonitorProvider {
    pub fn new() -> Self {
        Default::default()
    }
}

impl MonitorProvider for DisplayMonitorProvider {
    fn get_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        {
            let cache = self.cache.borrow();
            if cache.is_valid() {
                debug!("Using cached monitor information");
                return Ok(cache.data.clone());
            }
        }

        self.refresh_monitors()
    }

    fn refresh_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        debug!("Refreshing monitor information");
        let monitors = fetch_system_monitors()?;

        self.cache.borrow_mut().update(monitors.clone());

        Ok(monitors)
    }

    fn has_configuration_changed(&self) -> DwallResult<bool> {
        let current_monitors = fetch_system_monitors()?;
        let cache = self.cache.borrow();

        if current_monitors.len() != cache.data.len() {
            return Ok(true);
        }

        for device_path in current_monitors.keys() {
            if !cache.data.contains_key(device_path) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

thread_local! {
    /// Caches friendly names for monitors to avoid querying them multiple times
    static MONITOR_FRIENDLY_NAMES_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Queries monitor information from the system
fn fetch_system_monitors() -> DwallResult<HashMap<String, DisplayMonitor>> {
    debug!("Fetching monitor information from system");
    let mut monitors = HashMap::new();

    for (index, display_path) in query_display_paths()?.into_iter().enumerate() {
        let target_info = query_target_name(display_path.adapter_id, display_path.target_id)?;
        let device_path = target_info.monitorDevicePath.to_string();

        let friendly_name = if let Some(friendly_name) = MONITOR_FRIENDLY_NAMES_CACHE
            .with(|friendly_names| friendly_names.borrow().get(&device_path).cloned())
        {
            debug!(
                "Using cached friendly name for '{}': {}",
                device_path, friendly_name
            );
            friendly_name
        } else {
            let name = match query_device_friendly_name(&device_path, &GUID_DEVINTERFACE_MONITOR) {
                Ok(name) => name,
                Err(e) => {
                    warn!(error = %e, "Failed to get friendly name, using fallback");
                    format!("Display {}", index + 1)
                }
            };

            MONITOR_FRIENDLY_NAMES_CACHE.with(|friendly_names| {
                friendly_names
                    .borrow_mut()
                    .insert(device_path.clone(), name.clone());
            });

            name
        };

        monitors.insert(
            device_path.clone(),
            DisplayMonitor::new(device_path, friendly_name, Some(index as u32)),
        );
    }

    let monitor_count = monitors.len();
    info!("Found {} active monitor(s)", monitor_count);

    Ok(monitors)
}
