//! Windows registry auto-start management.

use dwall::{
    RegistryError, RegistryKey,
    utils::{WideStringExt, WideStringRead},
};
use windows::Win32::{
    Foundation::ERROR_FILE_NOT_FOUND,
    System::Registry::{KEY_QUERY_VALUE, KEY_WRITE, REG_SZ},
};

use crate::error::{DwallSettingsError, DwallSettingsResult};

/// Manages Windows registry auto-start settings
pub struct AutoStartProvider;

impl AutoStartProvider {
    const APP_NAME: &'static str = "Dwall";
    const KEY_PATH: &'static str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

    fn get_executable_path() -> DwallSettingsResult<&'static str> {
        crate::daemon::daemon_path()?
            .to_str()
            .ok_or_else(|| DwallSettingsError::Daemon("Daemon path is not valid UTF-8".into()))
    }

    /// Enables auto-start by adding the application to the registry
    pub fn enable() -> DwallSettingsResult<()> {
        let exe_path_str = Self::get_executable_path()?;
        let registry_key = RegistryKey::open(Self::KEY_PATH, KEY_WRITE)?;

        // The install path contains spaces ("Dwall Settings"), so the command line
        // stored in `Run` has to be quoted; an unquoted value is the classic
        // unquoted-path hijack.
        let wide_path = Vec::from_str(&format!("\"{exe_path_str}\""));
        let path_bytes = unsafe {
            std::slice::from_raw_parts(
                wide_path.as_ptr() as *const u8,
                wide_path.len() * std::mem::size_of::<u16>(),
            )
        };
        registry_key.set(Self::APP_NAME, REG_SZ, path_bytes)?;

        Ok(())
    }

    /// Disables auto-start by removing the application from the registry
    pub fn disable() -> DwallSettingsResult<()> {
        let registry_key = RegistryKey::open(Self::KEY_PATH, KEY_WRITE)?;
        registry_key.delete(Self::APP_NAME)?;
        Ok(())
    }

    /// Checks if auto-start is currently enabled
    pub fn is_enabled() -> DwallSettingsResult<bool> {
        let registry_key = RegistryKey::open(Self::KEY_PATH, KEY_QUERY_VALUE)?;

        let mut data_type = REG_SZ;
        let mut data: Vec<u16> = Vec::new();
        let mut data_size = 0;

        if let Err(RegistryError::Query(windows_error)) =
            registry_key.query(Self::APP_NAME, None, None, Some(&mut data_size))
        {
            if windows_error == ERROR_FILE_NOT_FOUND {
                return Ok(false);
            }
            return Err(RegistryError::Query(windows_error).into());
        }

        data.resize(data_size as usize, 0);
        match registry_key.query(
            Self::APP_NAME,
            Some(std::ptr::addr_of_mut!(data_type)),
            Some(data.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        ) {
            Ok(()) => {
                let exe_path_str = Self::get_executable_path()?;
                let command_str = data.to_string();
                // Accept both the quoted form written by current versions and the
                // bare path written by older ones.
                Ok(command_str == exe_path_str || command_str == format!("\"{exe_path_str}\""))
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
