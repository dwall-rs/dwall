use std::{ffi::CString, sync::LazyLock};

use dwall::error::RegistryError;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS},
        System::Registry::{
            RegCloseKey, RegDeleteValueA, RegOpenKeyExA, RegQueryValueExA, RegSetValueExA, HKEY,
            HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_WRITE, REG_SAM_FLAGS, REG_SZ,
        },
    },
};

use crate::{error::DwallSettingsResult, DAEMON_EXE_PATH};

/// Windows 自动启动应用的注册表管理器
pub struct AutoStartManager {
    app_name: LazyLock<CString>,
    key_path: LazyLock<CString>,
}

impl AutoStartManager {
    const APP_NAME: &'static str = "Dwall";
    const KEY_PATH: &'static str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    pub fn new() -> Self {
        Self {
            app_name: LazyLock::new(|| CString::new(Self::APP_NAME).unwrap()),
            key_path: LazyLock::new(|| CString::new(Self::KEY_PATH).unwrap()),
        }
    }

    /// 打开注册表键
    fn open_registry_key(&self, access: REG_SAM_FLAGS) -> Result<HKEY, RegistryError> {
        let mut hkey = HKEY::default();
        unsafe {
            let reg_result = RegOpenKeyExA(
                HKEY_CURRENT_USER,
                PCSTR(self.key_path.as_ptr() as *const u8),
                0,
                access,
                &mut hkey,
            );

            if reg_result != ERROR_SUCCESS {
                error!(
                    key_path = %self.key_path.to_str().unwrap_or(""),
                    error_code = ?reg_result,
                    "Failed to open registry key"
                );
                return Err(RegistryError::Open(reg_result));
            }

            trace!("Registry key opened successfully");
            Ok(hkey)
        }
    }

    /// 关闭注册表键
    fn close_registry_key(&self, hkey: HKEY) -> DwallSettingsResult<()> {
        let close_result = unsafe { RegCloseKey(hkey) };
        if close_result != ERROR_SUCCESS {
            error!(error_code = ?close_result, "Failed to close registry key");
            return Err(RegistryError::Close(close_result).into());
        }

        trace!("Registry key closed successfully");
        Ok(())
    }

    /// 启用自动启动
    pub fn enable_auto_start(&self) -> DwallSettingsResult<()> {
        info!("Enabling auto start");
        let exe_path_str = DAEMON_EXE_PATH.get().unwrap().to_str().unwrap_or_default();

        let hkey = self.open_registry_key(KEY_WRITE)?;

        unsafe {
            let set_result = RegSetValueExA(
                hkey,
                PCSTR(self.app_name.as_ptr() as *const u8),
                0,
                REG_SZ,
                Some(exe_path_str.as_bytes()),
            );

            if set_result != ERROR_SUCCESS {
                warn!(
                    app_name = %Self::APP_NAME,
                    path = %exe_path_str,
                    error_code = ?set_result,
                    "Failed to set registry value for auto start"
                );
                self.close_registry_key(hkey)?;
                return Err(RegistryError::Set(set_result).into());
            }
        }

        self.close_registry_key(hkey)?;
        info!("Auto start enabled successfully");
        Ok(())
    }

    /// 禁用自动启动
    pub fn disable_auto_start(&self) -> DwallSettingsResult<()> {
        info!("Disabling auto start");
        let hkey = self.open_registry_key(KEY_WRITE)?;

        unsafe {
            let delete_result = RegDeleteValueA(hkey, PCSTR(self.app_name.as_ptr() as *const u8));

            if delete_result != ERROR_SUCCESS {
                warn!(
                    app_name = %Self::APP_NAME,
                    error_code = ?delete_result,
                    "Failed to delete registry value for auto start"
                );
                self.close_registry_key(hkey)?;
                return Err(RegistryError::Delete(delete_result).into());
            }
        }

        self.close_registry_key(hkey)?;
        info!("Auto start disabled successfully");
        Ok(())
    }

    /// 检查是否启用自动启动
    pub fn check_auto_start(&self) -> DwallSettingsResult<bool> {
        trace!("Checking auto start status");
        let hkey = self.open_registry_key(KEY_QUERY_VALUE)?;

        let mut value_type = REG_SZ;
        let mut data: Vec<u8> = Vec::new();
        let mut data_size = 0;

        unsafe {
            // 首次调用获取数据大小
            let query_result = RegQueryValueExA(
                hkey,
                PCSTR(self.app_name.as_ptr() as *const u8),
                Some(std::ptr::null_mut()),
                Some(std::ptr::null_mut()),
                None,
                Some(&mut data_size),
            );

            match query_result {
                ERROR_SUCCESS => {
                    data.resize(data_size as usize, 0);
                    let second_query_result = RegQueryValueExA(
                        hkey,
                        PCSTR(self.app_name.as_ptr() as *const u8),
                        Some(std::ptr::null_mut()),
                        Some(&mut value_type),
                        Some(data.as_mut_ptr()),
                        Some(&mut data_size),
                    );

                    self.close_registry_key(hkey)?;

                    if second_query_result != ERROR_SUCCESS {
                        error!(
                            app_name = %Self::APP_NAME,
                            error_code = ?second_query_result,
                            "Failed to query registry value"
                        );
                        return Err(RegistryError::Query(second_query_result).into());
                    }

                    let command = String::from_utf8_lossy(&data);
                    let is_auto_start = command.contains("--auto-start");

                    debug!(
                        app_name = %Self::APP_NAME,
                        auto_start_status = is_auto_start,
                        "Auto start status retrieved"
                    );

                    Ok(is_auto_start)
                }
                ERROR_FILE_NOT_FOUND => {
                    debug!(
                        app_name = %Self::APP_NAME,
                        "No auto start entry found"
                    );
                    self.close_registry_key(hkey)?;
                    Ok(false)
                }
                _ => {
                    error!(
                        app_name = %Self::APP_NAME,
                        error_code = ?query_result,
                        "Unexpected error querying registry"
                    );
                    self.close_registry_key(hkey)?;
                    Err(RegistryError::Query(query_result).into())
                }
            }
        }
    }
}

#[tauri::command]
pub fn enable_auto_start(manager: tauri::State<AutoStartManager>) -> DwallSettingsResult<()> {
    manager.enable_auto_start()
}

#[tauri::command]
pub fn disable_auto_start(manager: tauri::State<AutoStartManager>) -> DwallSettingsResult<()> {
    manager.disable_auto_start()
}

#[tauri::command]
pub fn check_auto_start(manager: tauri::State<AutoStartManager>) -> DwallSettingsResult<bool> {
    manager.check_auto_start()
}
