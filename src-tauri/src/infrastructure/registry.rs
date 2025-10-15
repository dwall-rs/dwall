//! Registry infrastructure module
//!
//! This module contains Windows registry operations.

use dwall::{utils::string::WideStringRead, RegistryError, RegistryKey};
use windows::Win32::{
    Foundation::ERROR_FILE_NOT_FOUND,
    System::Registry::{KEY_QUERY_VALUE, KEY_WRITE, REG_SZ},
};

use crate::{error::DwallSettingsResult, DAEMON_EXE_PATH};

/// Manages Windows registry auto-start settings for an application
pub struct AutoStartManager;

impl AutoStartManager {
    /// Constant for the application name in the registry
    const APP_NAME: &'static str = "Dwall";
    /// Constant registry path for Windows auto-start entries
    const KEY_PATH: &'static str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    /// Retrieves the executable path for auto-start
    ///
    /// # Errors
    /// Returns an error if the executable path cannot be retrieved
    fn get_executable_path() -> &'static str {
        DAEMON_EXE_PATH.get().unwrap().to_str().unwrap()
    }

    /// Enables auto-start by adding the application to the registry
    ///
    /// # Errors
    /// Returns a `DwallSettingsResult` if auto-start cannot be enabled
    pub fn enable_auto_start() -> DwallSettingsResult<()> {
        // Safely get the executable path
        let exe_path_str = Self::get_executable_path();

        let registry_key = RegistryKey::open(Self::KEY_PATH, KEY_WRITE)?;
        registry_key.set(Self::APP_NAME, REG_SZ, exe_path_str.as_bytes())?;

        Ok(())
    }

    /// Disables auto-start by removing the application from the registry
    ///
    /// # Errors
    /// Returns a `DwallSettingsResult` if auto-start cannot be disabled
    pub fn disable_auto_start() -> DwallSettingsResult<()> {
        let registry_key = RegistryKey::open(Self::KEY_PATH, KEY_WRITE)?;
        registry_key.delete(Self::APP_NAME)?;

        Ok(())
    }

    /// Checks if auto-start is currently enabled
    ///
    /// # Errors
    /// Returns a `DwallSettingsResult` if the auto-start status cannot be determined
    pub fn check_auto_start() -> DwallSettingsResult<bool> {
        let registry_key = RegistryKey::open(Self::KEY_PATH, KEY_QUERY_VALUE)?;

        // Prepare variables for querying registry value
        let mut data_type = REG_SZ;
        let mut data: Vec<u16> = Vec::new();
        let mut data_size = 0;

        // First query to get required buffer size
        if let Err(RegistryError::Query(windows_error)) =
            registry_key.query(Self::APP_NAME, None, None, Some(&mut data_size))
        {
            if windows_error == ERROR_FILE_NOT_FOUND {
                return Ok(false);
            }

            return Err(RegistryError::Query(windows_error).into());
        }

        // Allocate buffer and perform second query
        data.resize(data_size as usize, 0);
        match registry_key.query(
            Self::APP_NAME,
            Some(std::ptr::addr_of_mut!(data_type)),
            Some(data.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        ) {
            Ok(()) => {
                // Get the current executable path
                let exe_path_str = Self::get_executable_path();

                let command_str = data.to_string();

                // Compare registry value with current executable path
                let is_auto_start = command_str == exe_path_str;

                Ok(is_auto_start)
            }
            Err(RegistryError::Query(err)) => {
                if err == ERROR_FILE_NOT_FOUND {
                    return Ok(false);
                }

                Err(RegistryError::Query(err).into())
            }
            _ => unreachable!(),
        }
    }
}
