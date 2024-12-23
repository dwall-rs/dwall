use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

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

use crate::error::DwallResult;

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

/// Windows registry helper utilities
struct RegistryHelper;

impl RegistryHelper {
    /// Convert a string to a wide character string (Windows-compatible)
    fn to_wide_string(s: &str) -> Vec<u16> {
        trace!("Converting string to wide string: {}", s);
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    /// Open a registry key with specified access rights
    fn open_key(path: &str, access: REG_SAM_FLAGS) -> DwallResult<HKEY> {
        debug!("Attempting to open registry key: {}", path);
        let wide_path = Self::to_wide_string(path);
        let mut hkey = HKEY::default();

        unsafe {
            let result = RegOpenKeyExW(
                HKEY_CURRENT_USER,
                PCWSTR(wide_path.as_ptr()),
                0,
                access,
                &mut hkey,
            );

            match result {
                ERROR_SUCCESS => {
                    info!("Successfully opened registry key: {}", path);
                    Ok(hkey)
                }
                err => {
                    error!("Failed to open registry key: {} (Error: {})", path, err.0);
                    Err(ColorModeRegistryError::Open(err.0).into())
                }
            }
        }
    }

    /// Close a previously opened registry key
    fn close_key(hkey: HKEY) -> DwallResult<()> {
        trace!("Attempting to close registry key");
        unsafe {
            match RegCloseKey(hkey) {
                ERROR_SUCCESS => {
                    debug!("Successfully closed registry key");
                    Ok(())
                }
                err => {
                    warn!("Failed to close registry key (Error: {})", err.0);
                    Err(ColorModeRegistryError::Close(err.0).into())
                }
            }
        }
    }
}

/// Color mode management utility
pub struct ColorModeManager;

impl ColorModeManager {
    const PERSONALIZE_KEY_PATH: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const APPS_THEME_VALUE: &str = "AppsUseLightTheme";
    const SYSTEM_THEME_VALUE: &str = "SystemUsesLightTheme";

    /// Retrieve the current system color mode from registry
    fn get_current_mode() -> DwallResult<ColorMode> {
        info!("Retrieving current system color mode");
        let hkey = RegistryHelper::open_key(Self::PERSONALIZE_KEY_PATH, KEY_QUERY_VALUE)?;

        let value_name = RegistryHelper::to_wide_string(Self::APPS_THEME_VALUE);
        let mut value: u32 = 0;
        let mut size = std::mem::size_of::<u32>() as u32;

        unsafe {
            let result = RegQueryValueExW(
                hkey,
                PCWSTR(value_name.as_ptr()),
                None,
                None,
                Some(&mut value as *mut u32 as *mut u8),
                Some(&mut size),
            );

            RegistryHelper::close_key(hkey)?;

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
                    error!("Failed to query color mode value (Error: {})", err.0);
                    Err(ColorModeRegistryError::Query(err.0).into())
                }
            }
        }
    }

    /// Set the system color mode in the registry
    pub fn set_color_mode(mode: ColorMode) -> DwallResult<()> {
        info!("Setting system color mode to {:?}", mode);
        let hkey = RegistryHelper::open_key(Self::PERSONALIZE_KEY_PATH, KEY_SET_VALUE)?;

        let value = match mode {
            ColorMode::Light => [1u8, 0, 0, 0],
            ColorMode::Dark => [0u8, 0, 0, 0],
        };

        let apps_value = RegistryHelper::to_wide_string(Self::APPS_THEME_VALUE);
        let system_value = RegistryHelper::to_wide_string(Self::SYSTEM_THEME_VALUE);

        unsafe {
            let set_apps_result = RegSetValueExW(
                hkey,
                PCWSTR(apps_value.as_ptr()),
                0,
                REG_DWORD,
                Some(&value),
            );

            let set_system_result = RegSetValueExW(
                hkey,
                PCWSTR(system_value.as_ptr()),
                0,
                REG_DWORD,
                Some(&value),
            );

            RegistryHelper::close_key(hkey)?;

            match (set_apps_result, set_system_result) {
                (ERROR_SUCCESS, ERROR_SUCCESS) => {
                    info!("Successfully set color mode to {:?}", mode);
                }
                _ => {
                    error!(
                        "Failed to set color mode (Apps result: {}, System result: {})",
                        set_apps_result.0, set_system_result.0
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

    trace!("Determining color mode with altitude: {}", altitude);

    if altitude > DAY_ALTITUDE_THRESHOLD {
        // Daytime: Sun is above the horizon
        trace!("Altitude above threshold, returning Light mode");
        ColorMode::Light
    } else if altitude < TWILIGHT_ALTITUDE_THRESHOLD {
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

pub fn set_color_mode(color_mode: ColorMode) -> DwallResult<()> {
    let current_color_mode = ColorModeManager::get_current_mode()?;
    if current_color_mode == color_mode {
        info!("Color mode is already set to {:?}", color_mode);
        return Ok(());
    }

    ColorModeManager::set_color_mode(color_mode)
}

fn notify_theme_change() -> DwallResult<()> {
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
                warn!("Failed to broadcast {}: {}", notification, e);
            }
        }
    }

    Ok(())
}
