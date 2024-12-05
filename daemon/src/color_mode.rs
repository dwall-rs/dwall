use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{ERROR_SUCCESS, LPARAM, WPARAM},
        System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
            KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD,
        },
        UI::WindowsAndMessaging::{SendNotifyMessageW, HWND_BROADCAST, WM_SETTINGCHANGE},
    },
};

use crate::error::{DwallResult, RegistryError};

#[derive(Debug, PartialEq)]
pub enum ColorMode {
    Light,
    Dark,
}

pub fn determine_color_mode(altitude: f64) -> ColorMode {
    // 定义白天和夜间的判断条件
    // 这里可以根据具体需求调整阈值
    const DAY_ALTITUDE_THRESHOLD: f64 = 0.0; // 太阳高度角大于0度视为白天
    const TWILIGHT_ALTITUDE_THRESHOLD: f64 = -6.0; // 太阳高度角低于-6度视为夜间

    if altitude > DAY_ALTITUDE_THRESHOLD {
        // 白天：太阳在地平线以上
        ColorMode::Light
    } else if altitude < TWILIGHT_ALTITUDE_THRESHOLD {
        // 夜间：太阳在地平线下较低
        ColorMode::Dark
    } else {
        // 黄昏/黎明阶段：可以根据具体需求选择模式
        // 这里可以添加更复杂的逻辑，比如根据日出日落时间更精确判断
        ColorMode::Dark // 或者可以根据具体需求选择 Light 或 Dark
    }
}

pub fn get_current_color_mode() -> DwallResult<ColorMode> {
    let key_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let apps_value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();

    unsafe {
        let mut hkey = HKEY(std::ptr::null_mut());
        let r = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path.as_ptr()),
            0,
            KEY_QUERY_VALUE,
            &mut hkey,
        );
        if r != ERROR_SUCCESS {
            return Err(RegistryError::Open(r).into());
        }

        let mut value: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;
        let r = RegQueryValueExW(
            hkey,
            PCWSTR(apps_value_name.as_ptr()),
            Some(std::ptr::null_mut()),
            Some(std::ptr::null_mut()),
            Some(&mut value as *mut u32 as *mut u8),
            Some(&mut size),
        );

        if r != ERROR_SUCCESS {
            return Err(RegistryError::Query(r).into());
        }

        let r = RegCloseKey(hkey);
        if r != ERROR_SUCCESS {
            return Err(RegistryError::Close(r).into());
        }

        Ok(if value == 1 {
            ColorMode::Light
        } else {
            ColorMode::Dark
        })
    }
}

pub fn set_color_mode(mode: ColorMode) -> DwallResult<()> {
    let current_mode = get_current_color_mode()?;
    if current_mode == mode {
        return Ok(());
    }

    let key_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
        .encode_utf16()
        .collect();
    let apps_value_name: Vec<u16> = "AppsUseLightTheme\0".encode_utf16().collect();
    let system_value_name: Vec<u16> = "SystemUsesLightTheme\0".encode_utf16().collect();

    unsafe {
        let hkey_ptr = std::ptr::null_mut();
        let mut hkey = HKEY(hkey_ptr);
        let r = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key_path.as_ptr()),
            0,
            KEY_SET_VALUE,
            &mut hkey,
        );
        if r != ERROR_SUCCESS {
            return Err(RegistryError::Open(r).into());
        }

        let value: [u8; 4] = match mode {
            ColorMode::Light => [1, 0, 0, 0],
            ColorMode::Dark => [0, 0, 0, 0],
        };

        let r = RegSetValueExW(
            hkey,
            PCWSTR(apps_value_name.as_ptr()),
            0,
            REG_DWORD,
            Some(&value),
        );
        if r != ERROR_SUCCESS {
            return Err(RegistryError::Set(r).into());
        }

        let r = RegSetValueExW(
            hkey,
            PCWSTR(system_value_name.as_ptr()),
            0,
            REG_DWORD,
            Some(&value),
        );
        if r != ERROR_SUCCESS {
            return Err(RegistryError::Set(r).into());
        }

        let r = RegCloseKey(hkey);
        if r != ERROR_SUCCESS {
            return Err(RegistryError::Close(r).into());
        }

        let theme_name: Vec<u16> = "ImmersiveColorSet\0".encode_utf16().collect();
        let _ = SendNotifyMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(theme_name.as_ptr() as isize),
        );
    }

    Ok(())
}
