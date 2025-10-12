use std::fmt;

use serde::Deserialize;
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Registry::{KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD},
    UI::WindowsAndMessaging::{SendNotifyMessageW, HWND_BROADCAST, WM_SETTINGCHANGE},
};

use crate::{error::DwallResult, registry::RegistryKey, utils::string::WideStringExt};

// Color mode threshold constants
const DAY_ALTITUDE_THRESHOLD: f64 = 0.0;
const TWILIGHT_ALTITUDE_THRESHOLD: f64 = -6.0;

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ColorMode {
    Light,
    Dark,
}

impl ColorMode {
    fn as_u32(&self) -> u32 {
        match self {
            ColorMode::Light => 1,
            ColorMode::Dark => 0,
        }
    }
    fn to_le_bytes(&self) -> [u8; 4] {
        self.as_u32().to_le_bytes()
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorMode::Light => write!(f, "Light"),
            ColorMode::Dark => write!(f, "Dark"),
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

        let mut data: u32 = 0;
        let mut data_size = std::mem::size_of_val(&data) as u32;
        let mut data_type = REG_DWORD;

        registry_key.query(
            Self::APPS_THEME_VALUE,
            Some(std::ptr::addr_of_mut!(data_type)),
            Some(std::ptr::addr_of_mut!(data) as *mut u8),
            Some(&mut data_size),
        )?;
        debug!(value = data, "Retrieved app theme value from registry");

        let mode = if data == 1 {
            debug!("Current color mode is Light");
            ColorMode::Light
        } else {
            debug!("Current color mode is Dark");
            ColorMode::Dark
        };
        Ok(mode)
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

        let value = mode.to_le_bytes();

        registry_key
            .set(Self::APPS_THEME_VALUE, REG_DWORD, &value)
            .map_err(|e| {
                error!(error =?e, "Failed to set apps theme value");
                e
            })?;
        info!(mode = %mode, "Successfully set apps theme value");

        registry_key
            .set(Self::SYSTEM_THEME_VALUE, REG_DWORD, &value)
            .map_err(|e| {
                error!(error =?e, "Failed to set system theme value");
                e
            })?;
        info!(mode = %mode, "Successfully set system theme value");

        notify_theme_change()?;

        Ok(())
    }
}

/// Determine color mode based on solar altitude angle
///
/// # Arguments
/// * `altitude` - Solar altitude angle in degrees
///
/// # Returns
/// - `ColorMode::Light` if the sun is above the horizon or in twilight
/// - `ColorMode::Dark` if the sun is below the twilight threshold
///
/// # Thresholds
/// - Day: altitude > 0° (sun above horizon)
/// - Twilight: -6° < altitude ≤ 0° (civil twilight)
/// - Night: altitude ≤ -6° (civil twilight threshold)
pub fn determine_color_mode(altitude: f64) -> ColorMode {
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
        let theme_name = Vec::from_str(notification);
        unsafe {
            if let Err(e) = SendNotifyMessageW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(theme_name.as_ptr() as isize),
            ) {
                warn!(notification = notification, error = %e, "Failed to broadcast notification");
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
