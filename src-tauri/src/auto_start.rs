use std::{env, ffi::CString, sync::LazyLock};

use dwall::error::RegistryError;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{
            RegCloseKey, RegDeleteValueA, RegOpenKeyExA, RegQueryValueExA, RegSetValueExA, HKEY,
            HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_WRITE, REG_SZ,
        },
    },
};

use crate::error::DwallSettingsResult;

static APP_NAME: LazyLock<CString> = LazyLock::new(|| CString::new("Dwall").unwrap());

static KEY_PATH: LazyLock<CString> =
    LazyLock::new(|| CString::new("Software\\Microsoft\\Windows\\CurrentVersion\\Run").unwrap());

#[tauri::command]
pub fn enable_auto_start() -> DwallSettingsResult<()> {
    // 获取当前可执行文件路径
    let exe_path = env::current_exe()?;
    let exe_path_str = exe_path.to_str().unwrap();

    unsafe {
        let mut hkey = HKEY::default();
        let r = RegOpenKeyExA(
            HKEY_CURRENT_USER,
            PCSTR(KEY_PATH.as_ptr() as *const u8),
            0,
            KEY_WRITE,
            &mut hkey,
        );
        if r != ERROR_SUCCESS {
            return Err(RegistryError::Open(r).into());
        }

        let command = format!("\"{}\" --auto-start", exe_path_str);

        let r = RegSetValueExA(
            hkey,
            PCSTR(APP_NAME.as_ptr() as *const u8),
            0,
            REG_SZ,
            Some(command.as_bytes()),
        );
        if r != ERROR_SUCCESS {
            close_key(hkey)?;
            return Err(RegistryError::Set(r).into());
        }

        close_key(hkey)?;
    }

    Ok(())
}

#[tauri::command]
pub fn disable_auto_start() -> DwallSettingsResult<()> {
    let mut hkey = HKEY::default();
    unsafe {
        let reg_result = RegOpenKeyExA(
            HKEY_CURRENT_USER,
            PCSTR(KEY_PATH.as_ptr() as *const u8),
            0,
            KEY_WRITE,
            &mut hkey,
        );
        if reg_result != ERROR_SUCCESS {
            return Err(RegistryError::Open(reg_result).into());
        }

        let delete_result = RegDeleteValueA(hkey, PCSTR(APP_NAME.as_ptr() as *const u8));
        if delete_result != ERROR_SUCCESS {
            close_key(hkey)?;
            return Err(RegistryError::Delete(delete_result).into());
        }

        close_key(hkey)?;
    }

    Ok(())
}

#[tauri::command]
pub fn check_auto_start() -> DwallSettingsResult<bool> {
    let mut hkey = HKEY::default();

    unsafe {
        let reg_result = RegOpenKeyExA(
            HKEY_CURRENT_USER,
            PCSTR(KEY_PATH.as_ptr() as *const u8),
            0,
            KEY_QUERY_VALUE,
            &mut hkey,
        );

        if reg_result != ERROR_SUCCESS {
            error!(path = ?KEY_PATH, "Failed to open registry");
            return Err(RegistryError::Open(reg_result).into());
        }

        let mut value_type = REG_SZ;
        let mut data: Vec<u8> = Vec::new();
        let mut data_size = 0;

        let query_result = RegQueryValueExA(
            hkey,
            PCSTR(APP_NAME.as_ptr() as *const u8),
            Some(std::ptr::null_mut()),
            Some(std::ptr::null_mut()),
            None,
            Some(&mut data_size),
        );

        if query_result != ERROR_SUCCESS {
            close_key(hkey)?;
            if query_result == ERROR_FILE_NOT_FOUND {
                return Ok(false);
            }

            error!(path = ?KEY_PATH, name = ?APP_NAME, "Failed to query registry");
            return Err(RegistryError::Query(query_result).into());
        }

        data.resize(data_size as usize, 0);
        let query_result = RegQueryValueExA(
            hkey,
            PCSTR(APP_NAME.as_ptr() as *const u8), // 键名
            Some(std::ptr::null_mut()),
            Some(&mut value_type),
            Some(data.as_mut_ptr()), // 获取数据
            Some(&mut data_size),
        );

        if query_result != ERROR_SUCCESS {
            close_key(hkey)?;
            return Err(RegistryError::Query(query_result).into());
        }

        close_key(hkey)?;

        let command = String::from_utf8_lossy(&data);
        if command.contains("--auto-start") {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

fn close_key(hkey: HKEY) -> DwallSettingsResult<()> {
    let r = unsafe { RegCloseKey(hkey) };
    if r != ERROR_SUCCESS {
        return Err(RegistryError::Close(r).into());
    }

    Ok(())
}
