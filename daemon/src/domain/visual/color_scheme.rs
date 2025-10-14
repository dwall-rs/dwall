use std::fmt;

use serde::Deserialize;
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Registry::{KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD},
    UI::WindowsAndMessaging::{SendNotifyMessageW, HWND_BROADCAST, WM_SETTINGCHANGE},
};

use crate::{
    error::DwallResult, infrastructure::platform::windows::registry_client::RegistryKey,
    utils::string::WideStringExt, Position,
};

// Civil twilight angle (when sun is 6° below horizon)
// This is the standard threshold used by macOS for theme switching
const CIVIL_TWILIGHT_ANGLE: f64 = -6.0;

// Hysteresis band to prevent frequent switching (±0.5° around threshold)
const HYSTERESIS_BAND: f64 = 0.5;

// Latitude boundary definitions
const ARCTIC_CIRCLE_LATITUDE: f64 = 66.5; // Arctic/Antarctic circle boundary
const HIGH_LATITUDE_BOUNDARY: f64 = 45.0; // Boundary for high latitude adjustments
const TROPIC_BOUNDARY: f64 = 23.5; // Tropic of Cancer/Capricorn

// Threshold adjustment ranges (in degrees)
const POLAR_BASE_THRESHOLD: f64 = -8.0; // Base threshold for polar regions
const POLAR_MAX_ADJUSTMENT: f64 = 4.0; // Maximum deepening for extreme polar latitudes (-8° to -12°)
const HIGH_LAT_MAX_ADJUSTMENT: f64 = 3.0; // Maximum deepening for high latitudes (up to -3°)
const TROPICAL_MAX_ADJUSTMENT: f64 = 1.5; // Maximum shallowing for tropical regions (up to +1.5°)

// Astronomical twilight boundaries (for clamping)
const MIN_THRESHOLD: f64 = -12.0; // Nautical twilight end (no longer useful for theme switching)
const MAX_THRESHOLD: f64 = -4.5; // Just before civil twilight (still somewhat bright)

#[derive(Debug, PartialEq, Deserialize, Copy, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    fn as_u32(&self) -> u32 {
        match self {
            ColorScheme::Light => 1,
            ColorScheme::Dark => 0,
        }
    }

    fn to_le_bytes(self) -> [u8; 4] {
        self.as_u32().to_le_bytes()
    }
}

impl fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorScheme::Light => write!(f, "Light"),
            ColorScheme::Dark => write!(f, "Dark"),
        }
    }
}

/// Dynamic threshold configuration based on location
#[derive(Debug, Clone)]
pub struct ThresholdConfig {
    /// Primary switching threshold (solar altitude angle in degrees)
    pub switch_threshold: f64,
}

impl ThresholdConfig {
    /// Calculate threshold based on geographic location
    pub fn from_location(position: &Position) -> Self {
        let switch_threshold = Self::calculate_twilight_threshold(position);

        Self { switch_threshold }
    }

    /// Calculate twilight threshold dynamically based on latitude
    ///
    /// Factors considered:
    /// - Higher latitudes have longer twilight due to shallow sun path
    /// - Tropical regions have short twilight due to steep sun path
    /// - Uses cosine function to model sun path angle relative to horizon
    ///
    /// # Latitude Zones
    /// - Tropical (0° to 23.5°): Steep sun path → shallower threshold (toward -4.5°)
    /// - Mid-latitude (23.5° to 45°): Standard civil twilight (-6°)
    /// - High latitude (45° to 66.5°): Shallow sun path → deeper threshold (toward -9°)
    /// - Polar (>66.5°): Very shallow path → deepest threshold (-8° to -12°)
    fn calculate_twilight_threshold(position: &Position) -> f64 {
        let lat_rad = position.latitude().abs().to_radians();

        // Base threshold (civil twilight)
        let mut threshold = CIVIL_TWILIGHT_ANGLE;

        // Calculate latitude factor using cosine
        // cos(0°) = 1.0 (equator), cos(90°) = 0.0 (pole)
        // This accurately reflects sun path angle relative to horizon
        let latitude_factor = lat_rad.cos();

        if position.latitude().abs() > ARCTIC_CIRCLE_LATITUDE {
            // Arctic/Antarctic circles: very shallow sun paths
            // Requires deeper threshold to handle polar day/night transitions
            // Normalize extreme_factor: 0 at 66.5°N, 1 at 90°N
            let extreme_factor =
                (position.latitude().abs() - ARCTIC_CIRCLE_LATITUDE) / TROPIC_BOUNDARY;

            // Transition from -8° to -12° (into nautical twilight range)
            threshold = POLAR_BASE_THRESHOLD - extreme_factor * POLAR_MAX_ADJUSTMENT;
        } else if position.latitude().abs() > HIGH_LATITUDE_BOUNDARY {
            // High latitudes: longer twilight periods due to shallow sun angle
            // Use latitude_factor for smooth transition based on sun path geometry
            let adjustment = (1.0 - latitude_factor) * HIGH_LAT_MAX_ADJUSTMENT;
            threshold = CIVIL_TWILIGHT_ANGLE - adjustment;
        } else if position.latitude().abs() < TROPIC_BOUNDARY {
            // Tropical regions: sun path nearly vertical, very short twilight
            // Can use shallower threshold as transition is rapid
            // Normalize tropical_factor: 1 at equator, 0 at tropics
            let tropical_factor = (TROPIC_BOUNDARY - position.latitude().abs()) / TROPIC_BOUNDARY;
            threshold = CIVIL_TWILIGHT_ANGLE + tropical_factor * TROPICAL_MAX_ADJUSTMENT;
        }
        // else: Mid-latitudes (23.5° to 45°) use default CIVIL_TWILIGHT_ANGLE

        // Clamp to reasonable astronomical range
        // Prevents extreme values that would be impractical for theme switching
        threshold = threshold.clamp(MIN_THRESHOLD, MAX_THRESHOLD);

        trace!(
            threshold = threshold,
            latitude = position.latitude(),
            latitude_factor = latitude_factor,
            "Calculated twilight threshold"
        );

        threshold
    }

    /// Create configuration with default values (when location is unavailable)
    pub fn default_config() -> Self {
        Self {
            switch_threshold: CIVIL_TWILIGHT_ANGLE,
        }
    }
}

/// Color scheme manager for Windows system theme management
pub(crate) struct ColorSchemeManager;

impl ColorSchemeManager {
    const PERSONALIZE_KEY_PATH: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const APPS_THEME_VALUE: &str = "AppsUseLightTheme";
    const SYSTEM_THEME_VALUE: &str = "SystemUsesLightTheme";

    /// Retrieve the current system color scheme from registry
    pub(crate) fn get_current_scheme() -> DwallResult<ColorScheme> {
        info!("Retrieving current system color scheme");
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

        let scheme = if data == 1 {
            debug!("Current color scheme is Light");
            ColorScheme::Light
        } else {
            debug!("Current color scheme is Dark");
            ColorScheme::Dark
        };
        Ok(scheme)
    }

    /// Set the system color scheme in the registry
    pub(crate) fn set_color_scheme(scheme: ColorScheme) -> DwallResult<()> {
        info!(scheme = %scheme, "Setting system color scheme");
        let registry_key = RegistryKey::open(Self::PERSONALIZE_KEY_PATH, KEY_SET_VALUE)?;

        let value = scheme.to_le_bytes();

        registry_key
            .set(Self::APPS_THEME_VALUE, REG_DWORD, &value)
            .map_err(|e| {
                error!(error = ?e, "Failed to set apps theme value");
                e
            })?;
        info!(scheme = %scheme, "Successfully set apps theme value");

        registry_key
            .set(Self::SYSTEM_THEME_VALUE, REG_DWORD, &value)
            .map_err(|e| {
                error!(error = ?e, "Failed to set system theme value");
                e
            })?;
        info!(scheme = %scheme, "Successfully set system theme value");

        notify_theme_change()?;
        Ok(())
    }
}

/// Determine color scheme with hysteresis to avoid frequent switching
///
/// # Arguments
/// * `altitude` - Solar apparent altitude angle in degrees (corrected for refraction)
/// * `current_scheme` - Current color scheme to apply hysteresis
/// * `config` - Threshold configuration based on location
///
/// # Returns
/// Color scheme considering hysteresis band around the threshold
///
/// # Logic
/// Uses symmetric hysteresis around a single threshold:
/// - When Light: switches to Dark when sun drops below (threshold - hysteresis)
/// - When Dark: switches to Light when sun rises above (threshold + hysteresis)
/// - This creates a 1° hysteresis band (±0.5°) to prevent oscillation
///
/// # Example
/// With threshold = -6° and hysteresis = 0.5°:
/// - Light→Dark transition: altitude < -6.5°
/// - Dark→Light transition: altitude > -5.5°
/// - Between -6.5° and -5.5°: maintains current state
pub(crate) fn determine_color_scheme_with_hysteresis(
    altitude: f64,
    current_scheme: &ColorScheme,
    config: &ThresholdConfig,
) -> ColorScheme {
    trace!(
        altitude = altitude,
        current_scheme = %current_scheme,
        threshold = config.switch_threshold,
        "Determining color scheme with hysteresis"
    );

    let threshold = config.switch_threshold;

    match current_scheme {
        ColorScheme::Light => {
            // Sunset: switch to Dark when sun drops below (threshold - hysteresis)
            let switch_point = threshold - HYSTERESIS_BAND;
            if altitude < switch_point {
                trace!(switch_point, "Sun below lower bound, switching to Dark");
                ColorScheme::Dark
            } else {
                trace!("Remaining in Light scheme");
                ColorScheme::Light
            }
        }
        ColorScheme::Dark => {
            // Sunrise: switch to Light when sun rises above (threshold + hysteresis)
            let switch_point = threshold + HYSTERESIS_BAND;
            if altitude > switch_point {
                trace!(switch_point, "Sun above upper bound, switching to Light");
                ColorScheme::Light
            } else {
                trace!("Remaining in Dark scheme");
                ColorScheme::Dark
            }
        }
    }
}

/// Set the system color scheme, checking first if it needs to be changed
pub(crate) fn set_color_scheme(color_scheme: ColorScheme) -> DwallResult<()> {
    let current_color_scheme = ColorSchemeManager::get_current_scheme()?;
    if current_color_scheme == color_scheme {
        info!(scheme = %color_scheme, "Color scheme is already set");
        return Ok(());
    }

    info!(from = %current_color_scheme, to = %color_scheme, "Changing color scheme");
    ColorSchemeManager::set_color_scheme(color_scheme)?;

    if !verify_theme_change(&color_scheme)? {
        warn!("Theme change may not have been applied correctly");
    }

    Ok(())
}

fn verify_theme_change(expected: &ColorScheme) -> DwallResult<bool> {
    std::thread::sleep(std::time::Duration::from_millis(100));
    let actual = ColorSchemeManager::get_current_scheme()?;
    Ok(&actual == expected)
}

/// Notify the system about theme changes
fn notify_theme_change() -> DwallResult<()> {
    trace!("Broadcasting theme change notifications");

    let lparam = Vec::from_str("ImmersiveColorSet");

    unsafe {
        SendNotifyMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(lparam.as_ptr() as isize),
        )?;
    }

    debug!("Notified system about theme change");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_config_tropical() {
        // Tropical region (Singapore: 1.3°N)
        let position = Position::from_raw_position(1.3, 0., 15.);
        let config = ThresholdConfig::from_location(&position);

        // Tropical regions should have shallower threshold (closer to -4.5°)
        assert!(config.switch_threshold > -6.0);
        assert!(config.switch_threshold <= -4.5);
    }

    #[test]
    fn test_threshold_config_high_latitude() {
        // High latitude (Stockholm: 59.3°N)
        let position = Position::from_raw_position(59.3, 0., 28.0);
        let config = ThresholdConfig::from_location(&position);

        // High latitudes should have deeper threshold
        assert!(config.switch_threshold < -6.5);
        assert!(config.switch_threshold >= -9.5);
    }

    #[test]
    fn test_threshold_config_polar() {
        // Polar region (Tromsø: 69.6°N)
        let position = Position::from_raw_position(69.6, 0., 10.0);
        let config = ThresholdConfig::from_location(&position);

        // Polar regions should have even deeper threshold
        assert!(config.switch_threshold <= -8.0);
        assert!(config.switch_threshold >= -12.0);
    }

    #[test]
    fn test_threshold_config_mid_latitude() {
        // Mid latitude (New York: 40.7°N)
        let position = Position::from_raw_position(40.7, 0., 10.0);
        let config = ThresholdConfig::from_location(&position);

        // Mid latitudes should stay close to civil twilight
        assert!(config.switch_threshold >= -7.0);
        assert!(config.switch_threshold <= -5.5);
    }

    #[test]
    fn test_hysteresis_sunset() {
        let config = ThresholdConfig::default_config();
        // Default threshold is -6°, hysteresis is 0.5°

        // Sunset scenario: Light → Dark
        // Should switch when dropping below -6.5°
        let altitude = -6.6;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Light, &config),
            ColorScheme::Dark
        );

        // At -6.0°, still above lower bound (-6.5°), stays Light
        let altitude = -6.0;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Light, &config),
            ColorScheme::Light
        );

        // At -6.4°, just above lower bound, stays Light
        let altitude = -6.4;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Light, &config),
            ColorScheme::Light
        );
    }

    #[test]
    fn test_hysteresis_sunrise() {
        let config = ThresholdConfig::default_config();
        // Default threshold is -6°, hysteresis is 0.5°

        // Sunrise scenario: Dark → Light
        // Should switch when rising above -5.5°
        let altitude = -5.4;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Dark, &config),
            ColorScheme::Light
        );

        // At -6.0°, still below upper bound (-5.5°), stays Dark
        let altitude = -6.0;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Dark, &config),
            ColorScheme::Dark
        );

        // At -5.6°, just below upper bound, stays Dark
        let altitude = -5.6;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Dark, &config),
            ColorScheme::Dark
        );
    }

    #[test]
    fn test_hysteresis_band() {
        let config = ThresholdConfig::default_config();

        // Within hysteresis band (-6.5° to -5.5°), maintains current mode
        let altitude = -6.0;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Light, &config),
            ColorScheme::Light
        );
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Dark, &config),
            ColorScheme::Dark
        );
    }

    #[test]
    fn test_extreme_altitudes() {
        let config = ThresholdConfig::default_config();

        // Noon: sun high in sky
        let altitude = 60.0;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Dark, &config),
            ColorScheme::Light
        );
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Light, &config),
            ColorScheme::Light
        );

        // Midnight: sun far below horizon
        let altitude = -45.0;
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Light, &config),
            ColorScheme::Dark
        );
        assert_eq!(
            determine_color_scheme_with_hysteresis(altitude, &ColorScheme::Dark, &config),
            ColorScheme::Dark
        );
    }

    #[test]
    fn test_hysteresis_prevents_oscillation() {
        let config = ThresholdConfig::default_config();

        // Simulate small fluctuations around threshold
        let altitudes = vec![-6.2, -6.0, -5.8, -6.0, -6.2];

        let mut scheme = ColorScheme::Light;
        for altitude in altitudes {
            scheme = determine_color_scheme_with_hysteresis(altitude, &scheme, &config);
        }

        // Should remain stable (Light) despite fluctuations
        assert_eq!(scheme, ColorScheme::Light);
    }

    #[test]
    fn test_color_scheme_display() {
        assert_eq!(format!("{}", ColorScheme::Light), "Light");
        assert_eq!(format!("{}", ColorScheme::Dark), "Dark");
    }

    #[test]
    fn test_color_scheme_conversion() {
        assert_eq!(ColorScheme::Light.as_u32(), 1);
        assert_eq!(ColorScheme::Dark.as_u32(), 0);
        assert_eq!(ColorScheme::Light.to_le_bytes(), [1, 0, 0, 0]);
        assert_eq!(ColorScheme::Dark.to_le_bytes(), [0, 0, 0, 0]);
    }
}
