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
use windows::{Win32::Foundation::ERROR_INSUFFICIENT_BUFFER, core::HRESULT};

use crate::DwallResult;
use crate::monitor::MonitorProvider;
use crate::utils::WideStringRead;

/// Attempts allowed when the display configuration changes mid-query.
const MAX_QUERY_CONFIG_ATTEMPTS: u8 = 3;

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
    let (paths, _) = query_config()?;

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
        // The function returns the failure code directly; `GetLastError` is not part
        // of its contract (`Error::from_win32` was also renamed to `from_thread` in
        // the windows 0.62 line).
        let win_error = WindowsError::from_hresult(HRESULT::from_nt(result));
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

fn query_config() -> DwallResult<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    // The display configuration can change between the size query and the config
    // query — that is exactly the hot-plug case this code detects — in which case
    // `QueryDisplayConfig` reports ERROR_INSUFFICIENT_BUFFER and the buffers have
    // to be sized again.
    for _ in 0..MAX_QUERY_CONFIG_ATTEMPTS {
        let (mut path_count, mut mode_count) = get_buffer_sizes()?;

        let mut paths = vec![Default::default(); path_count as usize];
        let mut modes = vec![Default::default(); mode_count as usize];

        let err = unsafe {
            QueryDisplayConfig(
                QDC_ALL_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                None,
            )
        };

        if err == ERROR_INSUFFICIENT_BUFFER {
            debug!("Display configuration changed while querying, retrying");
            continue;
        }

        if err != ERROR_SUCCESS {
            error!(error = ?err, "Failed to get display config");
            return Err(DisplayError::QueryDisplayConfig(err).into());
        }

        paths.truncate(path_count as usize);
        modes.truncate(mode_count as usize);
        debug!(path_count = path_count, "Got display count");

        return Ok((paths, modes));
    }

    error!(
        attempts = MAX_QUERY_CONFIG_ATTEMPTS,
        "Failed to query display configuration after repeated retries"
    );
    Err(DisplayError::QueryDisplayConfig(ERROR_INSUFFICIENT_BUFFER).into())
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

    // A null buffer asks the API for the size the property needs; the call itself fails
    // with ERROR_INSUFFICIENT_BUFFER, which is the expected answer. The fixed 512-byte
    // buffer this replaces made a longer friendly name fail with that same error and
    // fall back, and long names do exist.
    let mut required_size = 0;
    unsafe {
        let _ = SetupDiGetDeviceRegistryPropertyW(
            device_info_set.as_raw(),
            device_info_data,
            SPDRP_FRIENDLYNAME,
            None,
            None,
            Some(&mut required_size),
        );
    }

    // A zero size means the property is unavailable; the second call then fails and
    // reports it, so the caller still gets its fallback name.
    let mut buffer = vec![0u8; required_size as usize];
    let mut written = 0;

    unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            device_info_set.as_raw(),
            device_info_data,
            SPDRP_FRIENDLYNAME,
            None,
            Some(buffer.as_mut()),
            Some(&mut written),
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
        .take((written / 2) as usize)
        .map(|chunk| u16::from_le_bytes(*chunk))
        .collect();

    Ok(u16_data.to_string())
}

// ── Monitor provider ────────────────────────────────────────────────────────

/// Cache expiration time for the monitor list itself.
///
/// Only [`DisplayMonitorProvider::get_monitors`] is served from here (the wallpaper
/// setter shares this provider); the change check reads the configuration shape
/// instead of enumerating, so a hot-plug is not hidden behind this TTL.
const CACHE_EXPIRY_SECONDS: u64 = 300; // 5 minutes

/// Display monitor information with serialization support
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
    /// Shape of the display configuration `data` was read from, when known.
    shape: Option<DisplayConfigShape>,
    /// When `data` was last compared against the system.
    last_full_check: Instant,
}

impl MonitorInfoCache {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            updated_at: Instant::now(),
            expiry_duration: Duration::from_secs(CACHE_EXPIRY_SECONDS),
            shape: None,
            last_full_check: Instant::now(),
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

    /// Whether the monitors the daemon writes to have changed.
    ///
    /// The cycle calls this every iteration, and the previous version enumerated
    /// every display device each time — a full `QueryDisplayConfig` plus a
    /// `SetupDiGetDeviceInterfaceDetailW` per monitor, several thousand times a day
    /// — while the comment above the cache claimed the calls were reduced by 90%.
    /// The cheap shape query below answers the common case (nothing was plugged or
    /// unplugged) with one `GetDisplayConfigBufferSizes` call, and the full
    /// comparison still runs whenever the shape moves or the interval elapses.
    fn has_configuration_changed(&self) -> DwallResult<bool> {
        let shape = display_configuration_shape()?;
        let (cached_shape, since_full_check) = {
            let cache = self.cache.borrow();
            (cache.shape, cache.last_full_check.elapsed())
        };

        if !shape_forces_full_check(cached_shape, shape, since_full_check) {
            debug!(
                path_count = shape.path_count,
                mode_count = shape.mode_count,
                "Display configuration shape unchanged, skipping the enumeration"
            );
            return Ok(false);
        }

        let current_monitors = fetch_system_monitors()?;
        let mut cache = self.cache.borrow_mut();

        let changed = monitor_set_changed(&cache.data, &current_monitors);

        // Both are recorded whether or not anything changed: the shape is what the
        // next cheap check compares against, and without the timestamp every cycle
        // would enumerate again.
        cache.shape = Some(shape);
        cache.last_full_check = Instant::now();

        if changed {
            // Refresh the cache too: callers re-apply wallpapers right after this
            // check and would otherwise read the pre-hot-plug monitor list.
            cache.update(current_monitors);
        }

        Ok(changed)
    }
}

/// The display configuration as the cheap query reports it.
///
/// `GetDisplayConfigBufferSizes` counts the paths and modes without enumerating
/// them, so reading it costs one call instead of the whole `QueryDisplayConfig` +
/// SetupAPI walk. Attaching or detaching a monitor changes the path count, which is
/// the only change this daemon acts on: a wallpaper is addressed by device path, so
/// a resolution change on the same monitor needs no re-application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DisplayConfigShape {
    path_count: u32,
    mode_count: u32,
}

/// How long an unchanged shape may skip the full comparison.
///
/// A monitor swapped for one that leaves both counts identical is the case the
/// cheap check cannot see, so the full comparison still runs periodically.
const FULL_CONFIG_CHECK_INTERVAL: Duration = Duration::from_secs(60);

/// Reads the display configuration shape without enumerating the devices.
fn display_configuration_shape() -> DwallResult<DisplayConfigShape> {
    let (path_count, mode_count) = get_buffer_sizes()?;

    Ok(DisplayConfigShape {
        path_count,
        mode_count,
    })
}

/// Whether the cheap shape check has to be backed by the full comparison.
fn shape_forces_full_check(
    cached: Option<DisplayConfigShape>,
    current: DisplayConfigShape,
    since_full_check: Duration,
) -> bool {
    cached != Some(current) || since_full_check >= FULL_CONFIG_CHECK_INTERVAL
}

thread_local! {
    /// Caches friendly names for monitors to avoid querying them multiple times.
    ///
    /// Only *successful* lookups are remembered (see [`friendly_name_for`]) and the
    /// table is pruned to the monitors that are currently attached (see
    /// [`retain_attached_friendly_names`]), so it cannot grow with every display a
    /// machine has ever seen.
    static MONITOR_FRIENDLY_NAMES_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// The fallback shown for a monitor whose friendly name could not be read.
fn fallback_friendly_name(index: usize) -> String {
    format!("Display {}", index + 1)
}

/// The remembered friendly name of a monitor, if its lookup ever succeeded.
fn cached_friendly_name(device_path: &str) -> Option<String> {
    MONITOR_FRIENDLY_NAMES_CACHE.with(|names| names.borrow().get(device_path).cloned())
}

/// Remembers a friendly name that was read successfully.
fn remember_friendly_name(device_path: &str, name: &str) {
    MONITOR_FRIENDLY_NAMES_CACHE.with(|names| {
        names
            .borrow_mut()
            .insert(device_path.to_string(), name.to_string());
    });
}

/// Forgets the names of monitors that are no longer attached.
///
/// This is what keeps the table bounded: it only ever holds one entry per attached
/// monitor, instead of one per device path the machine has ever reported.
fn retain_attached_friendly_names(attached: &HashMap<String, DisplayMonitor>) {
    MONITOR_FRIENDLY_NAMES_CACHE.with(|names| {
        names
            .borrow_mut()
            .retain(|device_path, _| attached.contains_key(device_path));
    });
}

/// The friendly name to use for a monitor, querying it at most once.
///
/// A *failed* lookup is deliberately not remembered: the previous version cached the
/// `Display N` fallback for the life of the process, so a monitor whose name could not
/// be read once — a device that was still starting, a transient SetupAPI failure — was
/// labelled wrongly until the daemon restarted.
fn friendly_name_for(
    device_path: &str,
    index: usize,
    query: impl FnOnce() -> DwallResult<String>,
) -> String {
    if let Some(friendly_name) = cached_friendly_name(device_path) {
        debug!(
            "Using cached friendly name for '{}': {}",
            device_path, friendly_name
        );
        return friendly_name;
    }

    match query() {
        Ok(name) => {
            remember_friendly_name(device_path, &name);

            name
        }
        Err(e) => {
            warn!(error = %e, "Failed to get friendly name, using fallback");

            fallback_friendly_name(index)
        }
    }
}

/// Whether the attached monitors differ from the cached set.
///
/// A wallpaper is addressed by device path, so the *set* of paths is what matters: a
/// resolution change on the same monitor needs no re-application, while a monitor
/// swapped for another leaves a path that disappeared and one that is new.
fn monitor_set_changed(
    cached: &HashMap<String, DisplayMonitor>,
    current: &HashMap<String, DisplayMonitor>,
) -> bool {
    cached.len() != current.len()
        || current
            .keys()
            .any(|device_path| !cached.contains_key(device_path))
}

/// Queries monitor information from the system
fn fetch_system_monitors() -> DwallResult<HashMap<String, DisplayMonitor>> {
    debug!("Fetching monitor information from system");
    let mut monitors = HashMap::new();

    for (index, display_path) in query_display_paths()?.into_iter().enumerate() {
        let target_info = query_target_name(display_path.adapter_id, display_path.target_id)?;
        let device_path = target_info.monitorDevicePath.to_string();

        let friendly_name = friendly_name_for(&device_path, index, || {
            query_device_friendly_name(&device_path, &GUID_DEVINTERFACE_MONITOR)
        });

        monitors.insert(
            device_path.clone(),
            DisplayMonitor::new(device_path, friendly_name, Some(index as u32)),
        );
    }

    // Drop the names of monitors that went away, so the table cannot grow with every
    // display the machine has ever reported.
    retain_attached_friendly_names(&monitors);

    let monitor_count = monitors.len();
    debug!("Found {} active monitor(s)", monitor_count);

    Ok(monitors)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    fn shape(path_count: u32, mode_count: u32) -> DisplayConfigShape {
        DisplayConfigShape {
            path_count,
            mode_count,
        }
    }

    #[test]
    fn the_first_check_always_enumerates() {
        // Nothing has been cached yet, so the shape cannot be compared.
        assert!(shape_forces_full_check(None, shape(3, 8), Duration::ZERO));
    }

    #[test]
    fn an_unchanged_shape_within_the_interval_skips_the_enumeration() {
        assert!(!shape_forces_full_check(
            Some(shape(3, 8)),
            shape(3, 8),
            Duration::from_secs(1)
        ));
    }

    #[test]
    fn attaching_or_detaching_a_monitor_forces_the_enumeration() {
        // The path count is what changes on a hot-plug.
        assert!(shape_forces_full_check(
            Some(shape(2, 8)),
            shape(3, 8),
            Duration::from_secs(1)
        ));
    }

    #[test]
    fn a_changed_mode_count_forces_the_enumeration() {
        assert!(shape_forces_full_check(
            Some(shape(3, 8)),
            shape(3, 9),
            Duration::from_secs(1)
        ));
    }

    #[test]
    fn the_full_comparison_still_runs_once_the_interval_elapses() {
        // A monitor swapped for one with an identical shape is only visible to the
        // full comparison, so an unchanged shape must not skip it forever.
        assert!(shape_forces_full_check(
            Some(shape(3, 8)),
            shape(3, 8),
            FULL_CONFIG_CHECK_INTERVAL
        ));
        assert!(!shape_forces_full_check(
            Some(shape(3, 8)),
            shape(3, 8),
            FULL_CONFIG_CHECK_INTERVAL - Duration::from_millis(1)
        ));
    }

    #[test]
    fn the_monitor_cache_stops_serving_the_list_once_it_expires() {
        let mut cache = MonitorInfoCache::new();
        assert!(!cache.is_valid(), "an empty cache must never be served");

        cache.update(HashMap::from([(
            "\\\\?\\DISPLAY#1".to_string(),
            DisplayMonitor::new_test(
                "\\\\?\\DISPLAY#1".to_string(),
                "Display 1".to_string(),
                Some(0),
            ),
        )]));
        assert!(cache.is_valid());

        cache.expiry_duration = Duration::ZERO;
        assert!(!cache.is_valid(), "an expired cache must be refilled");
    }

    #[test]
    fn refreshing_the_list_keeps_the_shape_check_intact() {
        let mut cache = MonitorInfoCache::new();
        cache.shape = Some(shape(3, 8));
        let checked_at = cache.last_full_check;

        cache.update(HashMap::new());

        // `update` only swaps the data; the shape bookkeeping belongs to the change
        // check, and losing it here would make every cycle enumerate again.
        assert_eq!(cache.shape, Some(shape(3, 8)));
        assert_eq!(cache.last_full_check, checked_at);
    }

    // ── The monitor set comparison ──────────────────────────────────────────

    fn monitor_map(paths: &[&str]) -> HashMap<String, DisplayMonitor> {
        paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                (
                    (*path).to_string(),
                    DisplayMonitor::new_test(
                        (*path).to_string(),
                        format!("Display {}", index + 1),
                        Some(index as u32),
                    ),
                )
            })
            .collect()
    }

    #[test]
    fn an_unchanged_monitor_set_is_not_a_change() {
        let monitors = monitor_map(&["\\\\?\\DISPLAY#1", "\\\\?\\DISPLAY#2"]);

        assert!(!monitor_set_changed(&monitors, &monitors));
    }

    /// A monitor swapped for another leaves both a new path and a missing one, which is
    /// what the daemon has to notice to re-apply a wallpaper.
    #[test]
    fn a_swapped_monitor_is_detected() {
        let cached = monitor_map(&["\\\\?\\DISPLAY#1", "\\\\?\\DISPLAY#2"]);
        let swapped = monitor_map(&["\\\\?\\DISPLAY#1", "\\\\?\\DISPLAY#3"]);

        assert!(monitor_set_changed(&cached, &swapped));
    }

    #[test]
    fn attaching_a_monitor_is_detected() {
        let cached = monitor_map(&["\\\\?\\DISPLAY#1"]);
        let attached = monitor_map(&["\\\\?\\DISPLAY#1", "\\\\?\\DISPLAY#2"]);

        assert!(monitor_set_changed(&cached, &attached));
    }

    /// Pinning the current behaviour: a resolution change keeps the device path, and a
    /// wallpaper is addressed by device path, so nothing has to be re-applied.
    #[test]
    fn a_resolution_change_is_not_a_change() {
        let cached = monitor_map(&["\\\\?\\DISPLAY#1"]);
        let mut resized = monitor_map(&["\\\\?\\DISPLAY#1"]);
        resized.insert(
            "\\\\?\\DISPLAY#1".to_string(),
            DisplayMonitor::new_test(
                "\\\\?\\DISPLAY#1".to_string(),
                "Display 1 (renamed)".to_string(),
                Some(0),
            ),
        );

        assert!(!monitor_set_changed(&cached, &resized));
    }

    // ── The friendly-name cache ─────────────────────────────────────────────
    //
    // These drive the helpers directly: a real SetupAPI query needs a real display, and
    // what goes wrong here is bookkeeping, not enumeration.

    #[test]
    fn a_failed_friendly_name_lookup_is_not_remembered() {
        let path = "\\\\?\\DISPLAY#friendly-failure";

        let fallback = friendly_name_for(path, 1, || {
            Err(
                DisplayError::GetDeviceRegistryProperty(windows::core::Error::from_hresult(
                    windows::core::HRESULT::from_win32(5),
                ))
                .into(),
            )
        });

        assert_eq!(fallback, "Display 2");
        assert!(
            cached_friendly_name(path).is_none(),
            "caching the fallback made a monitor wrong for the life of the process"
        );

        // The next enumeration can still find the real name.
        let later = friendly_name_for(path, 1, || Ok("DELL U2720Q".to_string()));
        assert_eq!(later, "DELL U2720Q");
        assert_eq!(cached_friendly_name(path).as_deref(), Some("DELL U2720Q"));
    }

    #[test]
    fn a_known_friendly_name_is_never_queried_again() {
        let path = "\\\\?\\DISPLAY#friendly-cached";
        remember_friendly_name(path, "DELL U2720Q");

        let queried = Cell::new(false);
        let name = friendly_name_for(path, 0, || {
            queried.set(true);

            Ok("something else".to_string())
        });

        assert_eq!(name, "DELL U2720Q");
        assert!(!queried.get(), "the cached name must answer instead");
    }

    /// The bound is not a capacity: entries for monitors that are gone are dropped, so
    /// the table holds one name per attached display however long the machine runs.
    #[test]
    fn the_friendly_name_cache_forgets_monitors_that_are_gone() {
        let gone = "\\\\?\\DISPLAY#friendly-gone";
        let attached = "\\\\?\\DISPLAY#friendly-attached";
        remember_friendly_name(gone, "Old monitor");
        remember_friendly_name(attached, "Current monitor");

        retain_attached_friendly_names(&monitor_map(&[attached]));

        assert!(cached_friendly_name(gone).is_none());
        assert_eq!(
            cached_friendly_name(attached).as_deref(),
            Some("Current monitor")
        );
    }
}
