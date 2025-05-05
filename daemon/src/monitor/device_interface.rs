use std::mem;

use windows::{
    core::{Free, GUID, PCWSTR},
    Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiEnumDeviceInfo, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
        SetupDiGetDeviceInterfaceDetailW, SetupDiGetDeviceRegistryPropertyW, DIGCF_DEVICEINTERFACE,
        DIGCF_PRESENT, HDEVINFO, SPDRP_FRIENDLYNAME, SP_DEVICE_INTERFACE_DATA,
        SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA,
    },
};

use crate::{monitor::error::MonitorError, utils::string::WideStringRead, DwallResult};

/// RAII wrapper for HDEVINFO to ensure proper resource cleanup
struct DeviceInfoSet(HDEVINFO);

impl DeviceInfoSet {
    /// Creates a new DeviceInfoSet for the specified device GUID
    fn new(device_guid: &GUID) -> DwallResult<Self> {
        let device_info_set = unsafe {
            SetupDiGetClassDevsW(
                Some(device_guid),
                None,
                None,
                DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
            )
        }
        .map_err(|e| {
            error!(error = ?e, "Failed to get device info set");
            MonitorError::GetDeviceInfoSet(Some(e))
        })?;

        if device_info_set.is_invalid() {
            error!("Device info set handle is invalid");
            return Err(MonitorError::GetDeviceInfoSet(None).into());
        }

        Ok(Self(device_info_set))
    }

    /// Returns the raw HDEVINFO handle
    fn as_raw(&self) -> HDEVINFO {
        self.0
    }
}

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        unsafe { self.0.free() };
        debug!("DeviceInfoSet released");
    }
}

/// Query the friendly name of a device by its device path and GUID
pub fn query_device_friendly_name(device_path: &str, device_guid: &GUID) -> DwallResult<String> {
    debug!(
        "Querying device friendly name, device path: {}",
        device_path
    );

    // Use RAII pattern to ensure device_info_set is properly freed
    let device_info_set = DeviceInfoSet::new(device_guid)?;

    // Find the device interface matching the provided path
    let device_index = find_device_interface(&device_info_set, device_guid, device_path)?;

    // Get device info data for the found device
    let device_info_data = get_device_info(&device_info_set, device_index)?;

    // Get the friendly name of the device
    let friendly_name = get_device_friendly_name(&device_info_set, &device_info_data)?;

    Ok(friendly_name)
}

/// Find a device interface by its path
fn find_device_interface(
    device_info_set: &DeviceInfoSet,
    device_guid: &GUID,
    target_path: &str,
) -> DwallResult<u32> {
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
        debug!(index, "Checking device interface");

        if let Some(current_path) = get_device_path(device_info_set, &device_interface_data)? {
            debug!(current_path, target_path, "Comparing device paths");

            if current_path.eq_ignore_ascii_case(target_path) {
                debug!("Found matching device at index {}", index);
                return Ok(index);
            }
        }

        index += 1;
    }

    error!("Failed to find matching device for path: {}", target_path);
    Err(MonitorError::MatchDevice.into())
}

/// Get the device path from a device interface data
fn get_device_path(
    device_info_set: &DeviceInfoSet,
    device_interface_data: &SP_DEVICE_INTERFACE_DATA,
) -> DwallResult<Option<String>> {
    unsafe {
        // First call to get required buffer size
        let mut required_size = 0;
        let _ = SetupDiGetDeviceInterfaceDetailW(
            device_info_set.as_raw(),
            device_interface_data,
            None,
            0,
            Some(&mut required_size),
            None,
        );

        // Ensure minimum buffer size for the detail data structure
        let detail_data_size = mem::size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
        let buffer_size = required_size.max(detail_data_size);

        // Allocate buffer with appropriate size
        let mut buffer = vec![0u8; buffer_size as usize];
        let detail_data = buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
        (*detail_data).cbSize = detail_data_size;

        // Second call to get actual data
        if let Err(e) = SetupDiGetDeviceInterfaceDetailW(
            device_info_set.as_raw(),
            device_interface_data,
            Some(detail_data),
            buffer_size,
            None,
            None,
        ) {
            warn!(error = ?e, "Failed to get device interface detail");
            return Ok(None);
        }

        // Convert device path to string
        let device_path = PCWSTR((*detail_data).DevicePath.as_ptr());
        match device_path.to_string() {
            Ok(path) => Ok(Some(path)),
            Err(e) => {
                warn!(error = ?e, "Failed to convert device path to string");
                Ok(None)
            }
        }
    }
}

/// Get device info data for a device at the specified index
fn get_device_info(
    device_info_set: &DeviceInfoSet,
    device_index: u32,
) -> DwallResult<SP_DEVINFO_DATA> {
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
        error!("Failed to get device info for index {}", device_index);
        MonitorError::GetDeviceInfo(e)
    })?;

    Ok(device_info_data)
}

/// Get the friendly name of a device
fn get_device_friendly_name(
    device_info_set: &DeviceInfoSet,
    device_info_data: &SP_DEVINFO_DATA,
) -> DwallResult<String> {
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
            error!(error = ?e, "Failed to get device registry property");
            MonitorError::GetDeviceRegistryProperty(e)
        })?;
    }

    // Convert buffer to u16 array and then to string
    let u16_data: Vec<u16> = buffer
        .chunks_exact(2)
        .take((required_size / 2) as usize)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    // Convert to string, excluding null terminator
    let friendly_name = u16_data.to_string();

    debug!(friendly_name, "Got device friendly name");
    Ok(friendly_name)
}
