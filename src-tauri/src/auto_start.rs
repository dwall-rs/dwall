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

/// Manages Windows registry auto-start settings for an application
pub struct AutoStartManager {
    /// Name of the application in the registry
    app_name: LazyLock<CString>,
    /// Registry key path for auto-start entries
    key_path: LazyLock<CString>,
}

impl AutoStartManager {
    /// Constant for the application name in the registry
    const APP_NAME: &'static str = "Dwall";
    /// Constant registry path for Windows auto-start entries
    const KEY_PATH: &'static str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    /// Creates a new AutoStartManager instance
    pub fn new() -> Self {
        Self {
            app_name: LazyLock::new(|| {
                CString::new(Self::APP_NAME).expect("Failed to create CString for app name")
            }),
            key_path: LazyLock::new(|| {
                CString::new(Self::KEY_PATH)
                    .expect("Failed to create CString for registry key path")
            }),
        }
    }

    /// Opens the Windows registry key with specified access
    ///
    /// # Errors
    /// Returns a `RegistryError` if the key cannot be opened
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

            match reg_result {
                ERROR_SUCCESS => {
                    trace!(
                        path = self.key_path.to_str().unwrap_or("Invalid path"),
                        "Registry key opened successfully",
                    );
                    Ok(hkey)
                }
                _ => {
                    error!(
                        path = self.key_path.to_str().unwrap_or("Invalid path"), error_code = ?reg_result,
                        "Failed to open registry key",
                    );
                    Err(RegistryError::Open(reg_result))
                }
            }
        }
    }

    /// Safely closes an opened registry key
    ///
    /// # Errors
    /// Returns a `DwallSettingsResult` if the key cannot be closed
    fn close_registry_key(&self, hkey: HKEY) -> DwallSettingsResult<()> {
        let close_result = unsafe { RegCloseKey(hkey) };

        match close_result {
            ERROR_SUCCESS => {
                trace!("Registry key closed successfully");
                Ok(())
            }
            _ => {
                error!(error_code = close_result.0, "Failed to close registry key",);
                Err(RegistryError::Close(close_result).into())
            }
        }
    }

    /// Retrieves the executable path for auto-start
    ///
    /// # Errors
    /// Returns an error if the executable path cannot be retrieved
    fn get_executable_path(&self) -> &'static str {
        DAEMON_EXE_PATH.get().unwrap().to_str().unwrap()
    }

    /// Enables auto-start by adding the application to the registry
    ///
    /// # Errors
    /// Returns a `DwallSettingsResult` if auto-start cannot be enabled
    pub fn enable_auto_start(&self) -> DwallSettingsResult<()> {
        info!("Attempting to enable auto-start");

        // Safely get the executable path
        let exe_path_str = self.get_executable_path();

        // Open registry key with write permissions
        let hkey = self.open_registry_key(KEY_WRITE)?;

        // Attempt to set the registry value
        let set_result = unsafe {
            RegSetValueExA(
                hkey,
                PCSTR(self.app_name.as_ptr() as *const u8),
                0,
                REG_SZ,
                Some(exe_path_str.as_bytes()),
            )
        };

        // Handle set result and ensure key is closed
        match set_result {
            ERROR_SUCCESS => {
                self.close_registry_key(hkey)?;
                info!("Auto-start enabled successfully");
                Ok(())
            }
            _ => {
                // Attempt to close key even if set failed
                let _ = self.close_registry_key(hkey);

                warn!(
                    app_name = Self::APP_NAME,
                    path = exe_path_str,
                    error_code = ?set_result,
                    "Failed to set auto-start",
                );
                Err(RegistryError::Set(set_result).into())
            }
        }
    }

    /// Disables auto-start by removing the application from the registry
    ///
    /// # Errors
    /// Returns a `DwallSettingsResult` if auto-start cannot be disabled
    pub fn disable_auto_start(&self) -> DwallSettingsResult<()> {
        info!("Attempting to disable auto-start");

        // Open registry key with write permissions
        let hkey = self.open_registry_key(KEY_WRITE)?;

        // Attempt to delete the registry value
        let delete_result =
            unsafe { RegDeleteValueA(hkey, PCSTR(self.app_name.as_ptr() as *const u8)) };

        // Handle delete result and ensure key is closed
        match delete_result {
            ERROR_SUCCESS => {
                self.close_registry_key(hkey)?;
                info!("Auto-start disabled successfully");
                Ok(())
            }
            ERROR_FILE_NOT_FOUND => {
                // If the value doesn't exist, it's not an error
                self.close_registry_key(hkey)?;
                debug!("No existing auto-start entry found during disable");
                Ok(())
            }
            _ => {
                // Attempt to close key even if delete failed
                let _ = self.close_registry_key(hkey);

                warn!(
                    app_name = Self::APP_NAME,
                    error_code = delete_result.0,
                    "Failed to disable auto-start",
                );
                Err(RegistryError::Delete(delete_result).into())
            }
        }
    }

    /// Checks if auto-start is currently enabled
    ///
    /// # Errors
    /// Returns a `DwallSettingsResult` if the auto-start status cannot be determined
    pub fn check_auto_start(&self) -> DwallSettingsResult<bool> {
        trace!("Checking auto-start status");

        // Open registry key with query permissions
        let hkey = self.open_registry_key(KEY_QUERY_VALUE)?;

        // Prepare variables for querying registry value
        let mut value_type = REG_SZ;
        let mut data: Vec<u8> = Vec::new();
        let mut data_size = 0;

        unsafe {
            // First query to get required buffer size
            let first_query_result = RegQueryValueExA(
                hkey,
                PCSTR(self.app_name.as_ptr() as *const u8),
                Some(std::ptr::null_mut()),
                Some(std::ptr::null_mut()),
                None,
                Some(&mut data_size),
            );

            match first_query_result {
                ERROR_SUCCESS => {
                    // Allocate buffer and perform second query
                    data.resize(data_size as usize, 0);
                    let second_query_result = RegQueryValueExA(
                        hkey,
                        PCSTR(self.app_name.as_ptr() as *const u8),
                        Some(std::ptr::null_mut()),
                        Some(&mut value_type),
                        Some(data.as_mut_ptr()),
                        Some(&mut data_size),
                    );

                    // Close the key first
                    self.close_registry_key(hkey)?;

                    // Process query results
                    match second_query_result {
                        ERROR_SUCCESS => {
                            // Get the current executable path
                            let exe_path_str = self.get_executable_path();

                            // Compare registry value with current executable path
                            let command = data
                                .iter()
                                .take_while(|&&x| x != 0)
                                .cloned()
                                .collect::<Vec<u8>>();
                            let command_str = String::from_utf8_lossy(&command).to_string();

                            let is_auto_start = command_str == exe_path_str;

                            info!(
                                app_name = Self::APP_NAME,
                                status = is_auto_start,
                                command = %command_str,
                                "Auto-start status",
                            );

                            Ok(is_auto_start)
                        }
                        _ => {
                            error!(
                                app_name = Self::APP_NAME,
                                error_code = second_query_result.0,
                                "Failed to query registry value",
                            );
                            Err(RegistryError::Query(second_query_result).into())
                        }
                    }
                }
                ERROR_FILE_NOT_FOUND => {
                    // Close the key first
                    self.close_registry_key(hkey)?;

                    debug!("No auto-start entry found for {}", Self::APP_NAME);
                    Ok(false)
                }
                _ => {
                    // Close the key first
                    self.close_registry_key(hkey)?;

                    error!(
                        app_name = Self::APP_NAME,
                        error_code = first_query_result.0,
                        "Unexpected error querying registry",
                    );
                    Err(RegistryError::Query(first_query_result).into())
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
