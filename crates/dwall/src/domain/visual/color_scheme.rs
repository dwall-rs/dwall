use std::fmt;

use serde::Deserialize;
use time::OffsetDateTime;
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Registry::{KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD},
    UI::WindowsAndMessaging::{HWND_BROADCAST, SendNotifyMessageW, WM_SETTINGCHANGE},
};

use crate::{
    Position,
    domain::time::solar_calculator::{SunPosition, constants::ATMOSPHERIC_REFRACTION_MAX},
    error::DwallResult,
    infrastructure::platform::windows::registry_client::RegistryKey,
    utils::string::WideStringExt,
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
    #[inline]
    const fn as_u32(&self) -> u32 {
        match self {
            ColorScheme::Light => 1,
            ColorScheme::Dark => 0,
        }
    }

    #[inline]
    const fn to_le_bytes(self) -> [u8; 4] {
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
    switch_threshold: f64,
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
        debug!("Retrieving current system color scheme");
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
/// * `sun_pos` - Solar position calculator for astronomical computations
/// * `current_scheme` - Current color scheme to apply hysteresis
/// * `config` - Dynamic threshold configuration based on latitude (FIX 1)
pub(crate) fn determine_color_scheme_with_hysteresis(
    sun_pos: &SunPosition,
    current_scheme: &ColorScheme,
    config: &ThresholdConfig,
    local_time: &OffsetDateTime,
) -> ColorScheme {
    let latitude = sun_pos.latitude();
    let declination = sun_pos.solar_declination();
    let current_altitude = sun_pos.altitude();

    let base_threshold = config.switch_threshold;

    // 1. Calculate the day's solar altitude extremes (with atmospheric refraction compensation)
    let max_altitude = 90.0 - (latitude - declination).abs() + ATMOSPHERIC_REFRACTION_MAX;
    let min_altitude = (latitude + declination).abs() - 90.0 + ATMOSPHERIC_REFRACTION_MAX;

    // 2. Polar day/night fallback mechanism
    if min_altitude > base_threshold {
        // Polar day: the lowest point of the day is above the threshold, force Light scheme to avoid switching to Dark in bright conditions
        trace!("Polar day detected. Forcing Light scheme.");
        return ColorScheme::Light;
    } else if max_altitude < base_threshold {
        // Polar night: astronomical triggers are invalid, use local civil time directly
        let hour = local_time.hour();

        // Set local waking hours: e.g., 7 AM to 6 PM Light, otherwise Dark
        // TODO: make these 7 and 18 user-configurable in the future
        let is_waking_hours = (7..18).contains(&hour);

        trace!(
            hour = hour,
            "Polar night detected. Using local clock fallback."
        );

        return if is_waking_hours {
            ColorScheme::Light
        } else {
            ColorScheme::Dark
        };
    }

    // 3. Use different thresholds depending on current state to create true hysteresis
    let switch_point = match current_scheme {
        ColorScheme::Light => base_threshold - HYSTERESIS_BAND,
        ColorScheme::Dark => base_threshold + HYSTERESIS_BAND,
    };

    let scheme = if current_altitude > switch_point {
        ColorScheme::Light
    } else {
        ColorScheme::Dark
    };

    // Normal decision: absolute comparison after escaping the hysteresis trap
    debug!(
        altitude = current_altitude,
        switch_point = switch_point,
        base_threshold = base_threshold,
        current_scheme = %current_scheme,
        time = %local_time,
        "Evaluating standard twilight crossing."
    );

    scheme
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
    use time::Month;

    use super::*;
    use crate::domain::time::solar_calculator::SunPosition;

    fn sun_position(lat: f64, lon: f64, local: OffsetDateTime) -> SunPosition {
        let utc = local.utc().unwrap();
        SunPosition::new(lat, lon, utc)
    }

    fn threshold(lat: f64, lon: f64) -> ThresholdConfig {
        ThresholdConfig::from_location(&Position::from_raw_position(lat, lon, 0.))
    }

    #[test]
    fn test_threshold_mid_latitude() {
        // Huocheng: latitude 44.3, classified as mid-latitude (<45), uses default -6°
        let config = threshold(44.3037058, 80.9801647);
        assert!((config.switch_threshold - CIVIL_TWILIGHT_ANGLE).abs() < 0.01);
    }

    #[test]
    fn test_threshold_high_latitude() {
        // Paris: 48.87 > 45, high latitude adjustment
        let config = threshold(48.8728329, 2.3281715);
        // Expected threshold between -6 and -9
        assert!(config.switch_threshold < CIVIL_TWILIGHT_ANGLE);
        assert!(config.switch_threshold >= -9.0);
    }

    #[test]
    fn test_threshold_tropical() {
        // Near equator, tropical_factor≈1, threshold≈-4.5, clamped
        let config = threshold(0.0, 40.0);
        assert!(config.switch_threshold > CIVIL_TWILIGHT_ANGLE);
        assert!((config.switch_threshold - MAX_THRESHOLD).abs() < 0.01); // should be clamped to -4.5
    }

    #[test]
    fn test_threshold_polar() {
        // Construct latitude 78° inside Arctic Circle, triggers polar threshold
        let config = threshold(78.0, 0.0);
        assert!(config.switch_threshold <= POLAR_BASE_THRESHOLD);
        assert!(config.switch_threshold >= MIN_THRESHOLD);
    }

    // ============================================================
    // Tests for determine_color_scheme_with_hysteresis
    //
    // There is hysteresis around sunrise and sunset, so we cannot switch color scheme
    // directly based solely on sunrise/sunset times.
    // ============================================================

    fn test_location(
        lat: f64,
        lon: f64,
        year: u16,
        month: Month,
        day: u8,
        offset: &str,
        cases: &[((u8, u8, u8), ColorScheme)],
    ) {
        let config = threshold(lat, lon);
        for &((hour, minute, second), expected) in cases {
            let time = OffsetDateTime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                offset.parse().unwrap(),
            )
            .unwrap();
            let sun = sun_position(lat, lon, time);
            let actual = determine_color_scheme_with_hysteresis(
                &sun,
                &ColorScheme::Dark, // initial scheme set to Dark; does not affect core logic (except hysteresis, tests use fixed initial value)
                &config,
                &time,
            );
            assert_eq!(
                actual, expected,
                "Location ({lat}, {lon}) at {time}: expected {expected:?}, got {actual:?}"
            );
        }
    }

    #[test]
    fn test_huocheng_midday_night() {
        // Huocheng, Xinjiang, China
        test_location(
            44.3037058,
            80.9801647,
            2026,
            Month::May,
            15,
            "+08:00",
            &[
                ((6, 30, 0), ColorScheme::Dark),  // before sunrise, should be Dark
                ((7, 15, 0), ColorScheme::Light), // after sunrise, should be Light
                ((12, 0, 0), ColorScheme::Light), // noon, should be Light
                ((21, 0, 0), ColorScheme::Light), // before sunset, should be Light
                ((22, 40, 0), ColorScheme::Dark), // after sunset, should be Dark
                ((3, 0, 0), ColorScheme::Dark),   // late night, should be Dark
            ],
        );
    }

    #[test]
    fn test_greenland_midday_night() {
        // Kujalleq, Greenland
        test_location(
            61.,
            -45.,
            2026,
            Month::January,
            1,
            "-01:00",
            &[
                ((10, 00, 0), ColorScheme::Dark),  // before sunrise, should be Dark
                ((10, 45, 0), ColorScheme::Light), // after sunrise, should be Light
                ((12, 0, 0), ColorScheme::Light),  // noon, should be Light
                ((15, 0, 0), ColorScheme::Light),  // before sunset, should be Light
                ((18, 10, 0), ColorScheme::Dark), // after sunset, larger hysteresis at high latitudes
                ((23, 0, 0), ColorScheme::Dark),  // late night, should be Dark
            ],
        );
    }

    #[test]
    fn test_iceland_midday_night() {
        // Hornstrandir, Iceland
        test_location(
            66.3617958,
            -22.4390253,
            2026,
            Month::March,
            1,
            "+00:00",
            &[
                ((7, 30, 0), ColorScheme::Dark),   // before sunrise, should be Dark
                ((9, 00, 0), ColorScheme::Light),  // after sunrise, should be Light
                ((12, 0, 0), ColorScheme::Light),  // noon, should be Light
                ((18, 30, 0), ColorScheme::Light), // before sunset, should be Light
                ((18, 50, 0), ColorScheme::Light), // after sunset, should be Dark
                ((23, 0, 0), ColorScheme::Dark),   // late night, should be Dark
            ],
        );
    }

    #[test]
    fn test_paris_midday_night() {
        // 9th arrondissement of Paris, Île-de-France, France
        test_location(
            48.8728329,
            2.3281715,
            2026,
            Month::May,
            1,
            "+02:00",
            &[
                ((5, 40, 0), ColorScheme::Dark),   // before sunrise, should be Dark
                ((6, 35, 0), ColorScheme::Light),  // after sunrise, should be Light
                ((12, 0, 0), ColorScheme::Light),  // noon, should be Light
                ((21, 00, 0), ColorScheme::Light), // before sunset, should be Light
                ((21, 30, 0), ColorScheme::Light), // after sunset, should be Dark
                ((23, 0, 0), ColorScheme::Dark),   // late night, should be Dark
            ],
        );
    }

    #[test]
    fn test_kenya_midday_night() {
        // Garissa County, Kenya
        test_location(
            0.3488742,
            40.1127608,
            2026,
            Month::December,
            1,
            "+03:00",
            &[
                ((5, 20, 0), ColorScheme::Dark),   // before sunrise, should be Dark
                ((6, 3, 0), ColorScheme::Light),   // after sunrise, should be Light
                ((12, 0, 0), ColorScheme::Light),  // noon, should be Light
                ((18, 10, 0), ColorScheme::Light), // before sunset, should be Light
                ((19, 00, 0), ColorScheme::Dark),  // after sunset, should be Dark
                ((23, 0, 0), ColorScheme::Dark),   // late night, should be Dark
            ],
        );
    }
}
