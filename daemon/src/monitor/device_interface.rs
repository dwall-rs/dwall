use std::mem;

use windows::{
    core::{Free, PCWSTR},
    Win32::Devices::DeviceAndDriverInstallation::{
        SetupDiEnumDeviceInfo, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
        SetupDiGetDeviceInterfaceDetailW, SetupDiGetDeviceRegistryPropertyW, DIGCF_DEVICEINTERFACE,
        DIGCF_PRESENT, HDEVINFO, SPDRP_FRIENDLYNAME, SP_DEVICE_INTERFACE_DATA,
        SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA,
    },
};

use crate::{monitor::error::MonitorError, utils::string::WideStringRead, DwallResult};

pub fn query_device_friendly_name(
    device_path: &str,
    device_guid: &windows::core::GUID,
) -> DwallResult<String> {
    debug!(
        "Querying device friendly name, device path: {}",
        device_path
    );

    unsafe {
        let mut device_info_set = get_device_info_set(device_guid)?;
        let device_index = find_device_interface(&device_info_set, device_guid, device_path)?;
        let device_info_data = get_device_info(&device_info_set, device_index)?;
        let friendly_name = get_device_friendly_name(&device_info_set, &device_info_data)?;
        device_info_set.free();
        Ok(friendly_name)
    }
}

unsafe fn get_device_info_set(device_guid: &windows::core::GUID) -> DwallResult<HDEVINFO> {
    let device_info_set = unsafe {
        SetupDiGetClassDevsW(
            Some(device_guid),
            None,
            None,
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )
    }?;

    if device_info_set.is_invalid() {
        error!("Failed to get device info set");
        return Err(MonitorError::GetDeviceInfoSet.into());
    }

    Ok(device_info_set)
}

unsafe fn find_device_interface(
    device_info_set: &HDEVINFO,
    device_guid: &windows::core::GUID,
    target_path: &str,
) -> DwallResult<u32> {
    let mut device_interface_data = SP_DEVICE_INTERFACE_DATA {
        cbSize: mem::size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
        InterfaceClassGuid: windows::core::GUID::zeroed(),
        Flags: 0,
        Reserved: 0,
    };

    let mut index = 0;

    while unsafe {
        SetupDiEnumDeviceInterfaces(
            *device_info_set,
            None,
            device_guid,
            index,
            &mut device_interface_data,
        )
        .is_ok()
    } {
        debug!(index, "Checking device interface");

        if let Some(current_path) = get_device_path(device_info_set, &device_interface_data)? {
            debug!(current_path, target_path, "Comparing device paths",);

            if current_path.eq_ignore_ascii_case(target_path) {
                return Ok(index);
            }
        }

        index += 1;
    }

    error!("Failed to find matching device");
    Err(MonitorError::MatchDevice.into())
}

fn get_device_path(
    device_info_set: &HDEVINFO,
    device_interface_data: &SP_DEVICE_INTERFACE_DATA,
) -> DwallResult<Option<String>> {
    unsafe {
        let mut required_size = 0;
        let _ = SetupDiGetDeviceInterfaceDetailW(
            *device_info_set,
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
            *device_info_set,
            device_interface_data,
            Some(detail_data),
            buffer_size,
            None,
            None,
        ) {
            warn!(error = ?e, "Failed to get device interface detail");
            return Ok(None);
        }

        let device_path = PCWSTR((*detail_data).DevicePath.as_ptr());
        Ok(Some(device_path.to_string().unwrap()))
    }
}

unsafe fn get_device_info(
    device_info_set: &HDEVINFO,
    device_index: u32,
) -> DwallResult<SP_DEVINFO_DATA> {
    let mut device_info_data = SP_DEVINFO_DATA {
        cbSize: mem::size_of::<SP_DEVINFO_DATA>() as u32,
        ClassGuid: windows::core::GUID::zeroed(),
        DevInst: 0,
        Reserved: 0,
    };

    if unsafe {
        SetupDiEnumDeviceInfo(*device_info_set, device_index, &mut device_info_data).is_err()
    } {
        error!("Failed to get device info");
        return Err(MonitorError::GetDeviceInfo.into());
    }

    Ok(device_info_data)
}

fn get_device_friendly_name(
    device_info_set: &HDEVINFO,
    device_info_data: &SP_DEVINFO_DATA,
) -> DwallResult<String> {
    unsafe {
        let mut buffer = vec![0u8; 256];
        let mut required_size = 0;

        if SetupDiGetDeviceRegistryPropertyW(
            *device_info_set,
            device_info_data,
            SPDRP_FRIENDLYNAME,
            None,
            Some(buffer.as_mut()),
            Some(&mut required_size),
        )
        .is_ok()
        {
            let u16_data: Vec<u16> = buffer
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();

            let friendly_name = u16_data[..required_size as usize - 1].to_string();

            return Ok(friendly_name.to_string());
        }
    }

    error!("Failed to get device property");
    Err(MonitorError::GetFriendlyName.into())
}
