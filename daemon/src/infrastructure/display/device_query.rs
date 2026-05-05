//! Device interface query utilities for Windows

use std::mem;

use windows::{
    Win32::Devices::DeviceAndDriverInstallation::{
        DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, HDEVINFO, SP_DEVICE_INTERFACE_DATA,
        SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA, SPDRP_FRIENDLYNAME,
        SetupDiEnumDeviceInfo, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
        SetupDiGetDeviceInterfaceDetailW, SetupDiGetDeviceRegistryPropertyW,
    },
    core::{Free, GUID, PCWSTR},
};

use crate::{DwallResult, utils::string::WideStringRead};

use super::error::DisplayError;

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
pub(crate) fn query_device_friendly_name(
    device_path: &str,
    device_guid: &GUID,
) -> DwallResult<String> {
    debug!(
        device_path,
        guid = ?device_guid,
        "Querying device friendly name"
    );

    let device_info_set = DeviceInfoSet::new(device_guid)?;
    let device_index = find_device_by_path(&device_info_set, device_guid, device_path)?;
    debug!(device_index, "Found device at index");

    let device_info_data = get_device_info_data(&device_info_set, device_index)?;
    let friendly_name = get_device_friendly_name(&device_info_set, &device_info_data)?;

    debug!(
        device_path,
        friendly_name, "Successfully retrieved device friendly name"
    );

    Ok(friendly_name)
}

fn find_device_by_path(
    device_info_set: &DeviceInfoSet,
    device_guid: &GUID,
    target_path: &str,
) -> DwallResult<u32> {
    debug!(
        target_path,
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
            debug!(index, current_path, "Checking device path");

            if current_path.eq_ignore_ascii_case(target_path) {
                debug!(index = index, "Found matching device");
                return Ok(index);
            }
        }

        index += 1;
    }

    error!(target_path, "Failed to find device matching path");
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
    debug!(device_index, "Getting device info data");

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
            device_index,
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
        .chunks_exact(2)
        .take((required_size / 2) as usize)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    Ok(u16_data.to_string())
}
