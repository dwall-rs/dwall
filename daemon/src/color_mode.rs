use std::fmt;

use serde::Deserialize;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{ERROR_SUCCESS, LPARAM, WPARAM},
        System::Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
            KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD, REG_SAM_FLAGS,
        },
        UI::WindowsAndMessaging::{SendNotifyMessageW, HWND_BROADCAST, WM_SETTINGCHANGE},
    },
};

use crate::{error::DwallResult, utils::string::WideStringExt};

/// Custom error types for registry operations
#[derive(thiserror::Error, Debug)]
pub enum ColorModeRegistryError {
    #[error("Registry operation failed: Open key {0}")]
    Open(u32),

    #[error("Registry operation failed: Query value {0}")]
    Query(u32),

    #[error("Registry operation failed: Set value {0}")]
    Set(u32),

    #[error("Registry operation failed: Close key {0}")]
    Close(u32),
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ColorMode {
    Light,
    Dark,
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorMode::Light => write!(f, "Light"),
            ColorMode::Dark => write!(f, "Dark"),
        }
    }
}

/// RAII wrapper for Windows registry key handles
/// Automatically closes the key handle when dropped
struct RegistryKey {
    hkey: HKEY,
    path: String,
}

impl RegistryKey {
    /// Open a registry key with specified access rights
    fn open(path: &str, access: REG_SAM_FLAGS) -> DwallResult<Self> {
        debug!(path = path, "Attempting to open registry key");
        let wide_path = Vec::from_str(path);
        let mut hkey = HKEY::default();

        unsafe {
            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_path.as_ptr()),
                None,
                access,
                &mut hkey,
            );

            match result {
                ERROR_SUCCESS => {
                    info!(path = path, "Successfully opened registry key");
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
                    Err(ColorModeRegistryError::Open(err.0).into())
                }
            }
        }
    }

    /// Get the raw HKEY handle
    fn as_raw(&self) -> HKEY {
        self.hkey
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

/// Color mode management utility for Windows
pub struct ColorModeManager;

impl ColorModeManager {
    const PERSONALIZE_KEY_PATH: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const APPS_THEME_VALUE: &str = "AppsUseLightTheme";
    const SYSTEM_THEME_VALUE: &str = "SystemUsesLightTheme";

    /// Retrieve the current system color mode from registry
    ///
    /// # Returns
    /// - `Ok(ColorMode)` - The current system color mode
    /// - `Err(DwallError)` - If registry operations fail
    pub fn get_current_mode() -> DwallResult<ColorMode> {
        info!("Retrieving current system color mode");
        let registry_key = RegistryKey::open(Self::PERSONALIZE_KEY_PATH, KEY_QUERY_VALUE)?;

        let value_name = Vec::from_str(Self::APPS_THEME_VALUE);
        let mut value: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;

        unsafe {
            let result = RegQueryValueExW(
                registry_key.as_raw(),
                PCWSTR(value_name.as_ptr()),
                None,
                None,
                Some(&mut value as *mut u32 as *mut u8),
                Some(&mut size),
            );

            match result {
                ERROR_SUCCESS => {
                    let mode = if value == 1 {
                        debug!("Current color mode is Light");
                        ColorMode::Light
                    } else {
                        debug!("Current color mode is Dark");
                        ColorMode::Dark
                    };
                    Ok(mode)
                }
                err => {
                    error!(
                        value_name = Self::APPS_THEME_VALUE,
                        error_code = err.0,
                        "Failed to query color mode value"
                    );
                    Err(ColorModeRegistryError::Query(err.0).into())
                }
            }
        }
    }

    /// Set the system color mode in the registry
    ///
    /// # Arguments
    /// * `mode` - The color mode to set (Light or Dark)
    ///
    /// # Returns
    /// - `Ok(())` - If the color mode was set successfully
    /// - `Err(DwallError)` - If registry operations fail
    pub fn set_color_mode(mode: ColorMode) -> DwallResult<()> {
        info!(mode = %mode, "Setting system color mode");
        let registry_key = RegistryKey::open(Self::PERSONALIZE_KEY_PATH, KEY_SET_VALUE)?;

        let value = match mode {
            ColorMode::Light => [1u8, 0, 0, 0],
            ColorMode::Dark => [0u8, 0, 0, 0],
        };

        let apps_value = Vec::from_str(Self::APPS_THEME_VALUE);
        let system_value = Vec::from_str(Self::SYSTEM_THEME_VALUE);

        unsafe {
            let set_apps_result = RegSetValueExW(
                registry_key.as_raw(),
                PCWSTR(apps_value.as_ptr()),
                None,
                REG_DWORD,
                Some(&value),
            );

            let set_system_result = RegSetValueExW(
                registry_key.as_raw(),
                PCWSTR(system_value.as_ptr()),
                None,
                REG_DWORD,
                Some(&value),
            );

            match (set_apps_result, set_system_result) {
                (ERROR_SUCCESS, ERROR_SUCCESS) => {
                    info!(mode = %mode, "Successfully set color mode");
                }
                _ => {
                    error!(
                        mode = %mode,
                        apps_result = set_apps_result.0,
                        system_result = set_system_result.0,
                        "Failed to set color mode"
                    );
                    return Err(ColorModeRegistryError::Set(
                        set_apps_result.0 | set_system_result.0,
                    )
                    .into());
                }
            };

            notify_theme_change()?;
        }

        Ok(())
    }
}

pub fn determine_color_mode(altitude: f64) -> ColorMode {
    // Define day and night determination criteria
    // Thresholds can be adjusted based on specific requirements
    const DAY_ALTITUDE_THRESHOLD: f64 = 0.0; // Sun above horizon considered daytime
    const TWILIGHT_ALTITUDE_THRESHOLD: f64 = -6.0; // Sun below horizon considered nighttime

    trace!(altitude = altitude, "Determining color mode");

    if altitude > DAY_ALTITUDE_THRESHOLD {
        // Daytime: Sun is above the horizon
        trace!("Altitude above threshold, returning Light mode");
        ColorMode::Light
    } else if altitude <= TWILIGHT_ALTITUDE_THRESHOLD {
        // Nighttime: Sun is significantly below the horizon
        trace!("Altitude below threshold, returning Dark mode");
        ColorMode::Dark
    } else {
        // Twilight/dawn phase: can choose mode based on specific requirements
        // More complex logic can be added
        trace!("Twilight/dawn phase, defaulting to Light mode");
        ColorMode::Light
    }
}

/// Set the system color mode, checking first if it needs to be changed
///
/// # Arguments
/// * `color_mode` - The color mode to set (Light or Dark)
///
/// # Returns
/// - `Ok(())` - If the color mode was set successfully or was already set
/// - `Err(DwallError)` - If registry operations fail
pub fn set_color_mode(color_mode: ColorMode) -> DwallResult<()> {
    let current_color_mode = ColorModeManager::get_current_mode()?;
    if current_color_mode == color_mode {
        info!(mode = %color_mode, "Color mode is already set");
        return Ok(());
    }

    info!(from = %current_color_mode, to = %color_mode, "Changing color mode");
    ColorModeManager::set_color_mode(color_mode)
}

/// Notify the system about theme changes
///
/// This function broadcasts Windows messages to notify applications about theme changes
fn notify_theme_change() -> DwallResult<()> {
    debug!("Broadcasting theme change notifications");
    let notifications = [
        "ImmersiveColorSet",
        "WindowsThemeElement",
        "UserPreferenceChanged",
        "ThemeChanged",
    ];

    for notification in notifications {
        let theme_name: Vec<u16> = format!("{}\0", notification).encode_utf16().collect();
        unsafe {
            if let Err(e) = SendNotifyMessageW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(theme_name.as_ptr() as isize),
            ) {
                warn!(notification = notification, error = ?e, "Failed to broadcast notification");
            } else {
                debug!(
                    notification = notification,
                    "Successfully broadcast notification"
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_color_mode() {
        // Test daytime (sun above horizon)
        let altitude = 10.0;
        assert_eq!(determine_color_mode(altitude), ColorMode::Light);

        // Test nighttime (sun well below horizon)
        let altitude = -10.0;
        assert_eq!(determine_color_mode(altitude), ColorMode::Dark);

        // Test twilight (sun slightly below horizon)
        let altitude = -3.0;
        assert_eq!(determine_color_mode(altitude), ColorMode::Light);

        // Test edge cases
        assert_eq!(determine_color_mode(0.0), ColorMode::Light); // Exactly at horizon
        assert_eq!(determine_color_mode(-6.0), ColorMode::Dark); // Exactly at twilight threshold
    }

    #[test]
    fn test_color_mode_display() {
        assert_eq!(format!("{}", ColorMode::Light), "Light");
        assert_eq!(format!("{}", ColorMode::Dark), "Dark");
    }
}
