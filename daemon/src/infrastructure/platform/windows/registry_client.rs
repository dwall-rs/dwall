//! Windows Registry client for system configuration access

use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS, WIN32_ERROR},
        System::Registry::{
            RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExA, HKEY,
            HKEY_CURRENT_USER, REG_SAM_FLAGS, REG_VALUE_TYPE,
        },
    },
};

use crate::utils::string::WideStringExt;

/// Registry operation errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// Failed to open registry key
    #[error("Failed to open registry key: {0:?}")]
    Open(WIN32_ERROR),

    /// Failed to query registry value
    #[error("Failed to query registry value: {0:?}")]
    Query(WIN32_ERROR),

    /// Failed to set registry value
    #[error("Failed to set registry value: {0:?}")]
    Set(WIN32_ERROR),

    /// Failed to close registry handle
    #[error("Failed to close registry handle: {0:?}")]
    Close(WIN32_ERROR),

    /// Failed to delete registry key
    #[error("Failed to delete registry key: {0:?}")]
    Delete(WIN32_ERROR),
}

type RegistryResult<T> = Result<T, RegistryError>;

/// RAII wrapper for Windows registry key handles
pub struct RegistryKey {
    hkey: HKEY,
    path: String,
}

impl RegistryKey {
    /// Open a registry key with specified access rights
    pub fn open(path: &str, access: REG_SAM_FLAGS) -> RegistryResult<Self> {
        debug!(path = path, "Attempting to open registry key");
        let wide_path = Vec::from_str(path);
        let mut hkey = HKEY::default();

        let result = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_path.as_ptr()),
                None,
                access,
                &mut hkey,
            )
        };

        match result {
            ERROR_SUCCESS => {
                debug!(path = path, "Successfully opened registry key");
                Ok(Self {
                    hkey,
                    path: path.to_string(),
                })
            }
            err => {
                error!(
                    path = path,
                    error_code = err.0,
                    "Failed to open registry key"
                );
                Err(RegistryError::Open(err))
            }
        }
    }

    /// Query registry value
    pub fn query(
        &self,
        name: &str,
        data_type: Option<*mut REG_VALUE_TYPE>,
        data: Option<*mut u8>,
        data_size: Option<*mut u32>,
    ) -> RegistryResult<()> {
        trace!(
            path = self.path,
            name = name,
            "Attempting to query registry value"
        );

        let value_name_wide = Vec::from_str(name);
        let value_name = PCWSTR(value_name_wide.as_ptr());

        let result =
            unsafe { RegQueryValueExW(self.hkey, value_name, None, data_type, data, data_size) };

        match result {
            ERROR_SUCCESS => {
                debug!(pointer = ?data, "Retrieved data from registry");
                Ok(())
            }
            _ => {
                error!(
                    name = name,
                    error_code = result.0,
                    "Failed to query registry value",
                );
                Err(RegistryError::Query(result))
            }
        }
    }

    /// Set a registry value
    pub fn set(&self, name: &str, value_type: REG_VALUE_TYPE, data: &[u8]) -> RegistryResult<()> {
        let value_name = PCSTR(name.as_ptr());

        unsafe {
            let result = RegSetValueExA(self.hkey, value_name, None, value_type, Some(data));

            match result {
                ERROR_SUCCESS => {
                    debug!(name = name, "Successfully set registry value");
                    Ok(())
                }
                err => {
                    error!(
                        name = name,
                        error_code = err.0,
                        "Failed to set registry value"
                    );
                    Err(RegistryError::Set(err))
                }
            }
        }
    }

    /// Delete a registry value
    pub fn delete(&self, name: &str) -> RegistryResult<()> {
        let value_name_wide = Vec::from_str(name);
        let value_name = PCWSTR(value_name_wide.as_ptr());

        unsafe {
            let result = RegDeleteValueW(self.hkey, value_name);

            match result {
                ERROR_SUCCESS => {
                    debug!(name = name, "Successfully deleted registry value");
                    Ok(())
                }
                ERROR_FILE_NOT_FOUND => {
                    warn!(name = name, "Registry value not found, skipping deletion");
                    Ok(())
                }
                err => {
                    error!(
                        name = name,
                        error_code = err.0,
                        "Failed to delete registry value"
                    );
                    Err(RegistryError::Delete(err))
                }
            }
        }
    }
}

impl Drop for RegistryKey {
    fn drop(&mut self) {
        trace!(path = self.path, "Automatically closing registry key");
        unsafe {
            let err = RegCloseKey(self.hkey);
            if err != ERROR_SUCCESS {
                warn!(
                    path = self.path,
                    error_code = err.0,
                    "Failed to close registry key on drop"
                );
            } else {
                debug!(path = self.path, "Successfully closed registry key");
            }
        }
    }
}
