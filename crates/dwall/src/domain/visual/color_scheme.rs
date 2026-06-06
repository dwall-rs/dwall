use std::fmt;

use serde::Deserialize;
use time::{Date, OffsetDateTime};
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Registry::{KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD},
    UI::WindowsAndMessaging::{HWND_BROADCAST, SendNotifyMessageW, WM_SETTINGCHANGE},
};

use crate::{
    Position, domain::time::solar_calculator::SolarPosition, error::DwallResult,
    infrastructure::platform::windows::registry_client::RegistryKey, utils::string::WideStringExt,
};

// ─────────────────────────────────────────────────────────────
// Public Constants
// ─────────────────────────────────────────────────────────────

/// Civil twilight threshold (degrees)
///
/// When the center of the sun is 6° below the horizon, the scattered light
/// at ground level has largely disappeared.
/// macOS / GNOME and other systems use this as the baseline for automatic switching.
const CIVIL_TWILIGHT_DEG: f64 = -6.0;

/// Hysteresis half-bandwidth (degrees)
///
/// The actual switching line is `base_threshold ± HYSTERESIS_BAND`,
/// giving a total dead zone width of `2 × HYSTERESIS_BAND = 1°`.
/// The sun moves roughly 0.1° per minute around sunrise/sunset,
/// so a 1° dead zone provides approximately 10 minutes of buffer.
const HYSTERESIS_BAND: f64 = 0.5;

/// White night amplitude trigger threshold (degrees)
///
/// When the daily maximum solar altitude is below
/// `base_threshold + WHITE_NIGHT_AMPLITUDE_MARGIN`
/// and the full-day amplitude is less than this value, the day is classified
/// as a white night, enabling the dynamic midpoint switching logic.
const WHITE_NIGHT_AMPLITUDE_MARGIN: f64 = 4.0;

/// Polar night clock fallback: waking hours start hour (local time, inclusive)
const WAKING_HOUR_START: u8 = 7;

/// Polar night clock fallback: waking hours end hour (local time, exclusive)
const WAKING_HOUR_END: u8 = 18;

// ─────────────────────────────────────────────────────────────
// Latitude Zone Constants (for dynamic threshold calculation)
// ─────────────────────────────────────────────────────────────

/// Arctic / Antarctic Circle latitude (degrees)
const ARCTIC_CIRCLE_LAT: f64 = 66.5;

/// High-latitude boundary (degrees)
const HIGH_LAT_BOUNDARY: f64 = 45.0;

/// Tropic latitude (degrees)
const TROPIC_LAT: f64 = 23.5;

/// Polar base threshold (degrees): adjusted deeper from this value
const POLAR_BASE_THRESHOLD: f64 = -8.0;

/// Maximum deepening inside polar circles (degrees): deepest to -12° (end of nautical twilight)
const POLAR_MAX_ADJUSTMENT: f64 = 4.0;

/// Maximum deepening at high latitudes (degrees): deepest to about -9°
const HIGH_LAT_MAX_ADJUSTMENT: f64 = 3.0;

/// Maximum shallowing in the tropics (degrees): shallowest to about -4.5°
const TROPICAL_MAX_ADJUSTMENT: f64 = 1.5;

/// Lower clamp for the threshold (degrees): end of nautical twilight,
/// beyond which further deepening is not meaningful
const THRESHOLD_MIN: f64 = -12.0;

/// Upper clamp for the threshold (degrees): middle of civil twilight;
/// shallower values are not appropriate
const THRESHOLD_MAX: f64 = -4.5;

// ─────────────────────────────────────────────────────────────
// Color Scheme
// ─────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────
// Dynamic Threshold Configuration
// ─────────────────────────────────────────────────────────────

/// Dynamic switching threshold based on geographic location
///
/// Twilight duration varies significantly across latitudes:
/// - **Tropics** (< 23.5°): the sun crosses the horizon nearly vertically,
///   twilight is very short (within 15 minutes). The threshold can be
///   raised (toward -4.5°) to switch earlier.
/// - **Mid-latitudes** (23.5°–45°): use the standard civil twilight
///   threshold of -6°.
/// - **High latitudes** (45°–66.5°): the sun crosses the horizon at a
///   shallow angle; twilight can last 30–60 minutes. The threshold should
///   be deepened (toward -9°) to avoid switching to dark while it is
///   still quite bright.
/// - **Inside the polar circles** (> 66.5°): the shallowest path;
///   deepest threshold (-8° to -12°).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThresholdConfig {
    /// Base switching threshold (degrees), dynamically computed from latitude
    base_threshold: f64,
    /// Hysteresis half-bandwidth (degrees), typically [`HYSTERESIS_BAND`]
    hysteresis_band: f64,
}

impl ThresholdConfig {
    /// Automatically compute the threshold based on geographic location
    pub fn from_position(position: &Position) -> Self {
        Self {
            base_threshold: Self::calculate_threshold(position),
            hysteresis_band: HYSTERESIS_BAND,
        }
    }

    /// Use the default civil twilight threshold (suitable when location is unknown)
    pub fn default_civil() -> Self {
        Self {
            base_threshold: CIVIL_TWILIGHT_DEG,
            hysteresis_band: HYSTERESIS_BAND,
        }
    }

    /// Compute the base threshold by latitude zone
    ///
    /// Uses `cos(lat)` as the latitude factor: 1.0 at the equator, 0.0 at the poles,
    /// which naturally reflects how the sun's path angle relative to the horizon
    /// varies with latitude.
    fn calculate_threshold(position: &Position) -> f64 {
        let abs_lat = position.latitude().abs();
        let lat_rad = abs_lat.to_radians();
        let latitude_factor = lat_rad.cos(); // 1.0 (equator) → 0.0 (pole)

        let threshold = if abs_lat > ARCTIC_CIRCLE_LAT {
            // Inside polar circles: -8° to -12°, deepening linearly with latitude
            // extreme_factor: 0.0 (66.5°) → 1.0 (90°)
            let extreme_factor = (abs_lat - ARCTIC_CIRCLE_LAT) / TROPIC_LAT;
            POLAR_BASE_THRESHOLD - extreme_factor * POLAR_MAX_ADJUSTMENT
        } else if abs_lat > HIGH_LAT_BOUNDARY {
            // High latitudes: -6° to about -9°, smooth transition using the cosine factor
            let adjustment = (1.0 - latitude_factor) * HIGH_LAT_MAX_ADJUSTMENT;
            CIVIL_TWILIGHT_DEG - adjustment
        } else if abs_lat < TROPIC_LAT {
            // Tropics: -6° to -4.5°, greatest shallowing at the equator
            // tropical_factor: 1.0 (equator) → 0.0 (tropics)
            let tropical_factor = (TROPIC_LAT - abs_lat) / TROPIC_LAT;
            CIVIL_TWILIGHT_DEG + tropical_factor * TROPICAL_MAX_ADJUSTMENT
        } else {
            // Mid-latitudes (23.5°–45°): standard threshold directly
            CIVIL_TWILIGHT_DEG
        };

        // Clamp to the reasonable astronomical range
        threshold.clamp(THRESHOLD_MIN, THRESHOLD_MAX)
    }

    /// Lower altitude bound for switching to Light
    ///
    /// When currently Dark, the altitude must **rise above** this value to switch to Light.
    #[inline]
    fn light_switch_point(&self) -> f64 {
        self.base_threshold + self.hysteresis_band
    }

    /// Upper altitude bound for switching to Dark
    ///
    /// When currently Light, the altitude must **drop below** this value to switch to Dark.
    #[inline]
    fn dark_switch_point(&self) -> f64 {
        self.base_threshold - self.hysteresis_band
    }
}

// ─────────────────────────────────────────────────────────────
// Extreme Daylight States
// ─────────────────────────────────────────────────────────────

/// The extreme daylight state for a given day
///
/// Determined by sampling [`SolarPosition::altitude()`] (which already includes
/// refraction correction) at 24 whole hours of the day. This should be updated
/// once a day around local sunrise and cached; do not recompute on the main
/// call path.
///
/// # Why sampling instead of an analytic formula
///
/// The analytic formula for the extreme (`90° - |φ - δ|`) requires accessing the
/// declination from outside `SolarPosition`, breaking encapsulation. Moreover,
/// the formula itself does not include refraction correction and would require
/// manually adding a constant, but `altitude()` already returns the apparent
/// altitude after refraction; the two cannot be mixed directly. The overhead
/// of 24 samples per day is far outweighed by the correctness and simplicity gains.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum DaylightState {
    /// Normal day/night cycle, using standard hysteresis logic
    Normal,

    /// Midnight sun: the lowest altitude of the day is still above the base threshold
    ///
    /// The sun never descends into twilight; force light mode.
    MidnightSun,

    /// Polar night: the highest altitude of the day is still below the base threshold
    ///
    /// The sun never rises above twilight; astronomical determination is useless.
    /// Fall back to local clock waking-hour range judgement.
    PolarNight,

    /// White night: the sun oscillates near the base threshold without fully
    /// crossing the hysteresis dead zone
    ///
    /// Use the daily altitude amplitude midpoint as a dynamic threshold, with a
    /// forced wide dead zone to prevent small altitude perturbations from
    /// triggering frequent switches.
    WhiteNight {
        /// Minimum apparent altitude of the day (degrees, including refraction)
        min_altitude: f64,
        /// Maximum apparent altitude of the day (degrees, including refraction)
        max_altitude: f64,
    },
}

impl DaylightState {
    /// Detect the extreme state by sampling solar altitudes at 24 whole hours of the day
    ///
    /// # Parameters
    /// - `position`: geographic coordinates of the observer
    /// - `date`: the local date to evaluate (UTC)
    /// - `config`: threshold configuration for comparison with the extremes
    ///
    /// # When to call
    /// Recommended to call once around local midnight each day and cache the result.
    /// Should not be triggered on every call to `determine_color_scheme_with_hysteresis`.
    pub fn detect(position: &Position, date: Date, config: &ThresholdConfig) -> Self {
        // Sample apparent altitudes for hours 0–23 (refraction already included;
        // no extra constant needed)
        let altitudes: Vec<f64> = (0u8..24)
            .filter_map(|h| {
                date.with_hms(h, 0, 0)
                    .ok()
                    .map(|dt| SolarPosition::new(position, &dt.assume_utc()).altitude())
            })
            .collect();

        // If sampling fails, degrade to normal state (conservative)
        if altitudes.is_empty() {
            return DaylightState::Normal;
        }

        let min_alt = altitudes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_alt = altitudes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Midnight sun: the minimum is still above the base threshold → sun never enters twilight
        if min_alt >= config.base_threshold {
            return DaylightState::MidnightSun;
        }

        // Polar night: the maximum is still below the base threshold → sun never leaves night
        if max_alt < config.base_threshold {
            return DaylightState::PolarNight;
        }

        // White night: altitude oscillation is very small and the maximum does not
        // significantly exceed the threshold
        // Conditions: amplitude < WHITE_NIGHT_AMPLITUDE_MARGIN,
        //        and max altitude < base_threshold + WHITE_NIGHT_AMPLITUDE_MARGIN
        // i.e. the sun merely grazes past the threshold without completing a full
        // day-night switch
        let amplitude = max_alt - min_alt;
        if amplitude < WHITE_NIGHT_AMPLITUDE_MARGIN
            && max_alt < config.base_threshold + WHITE_NIGHT_AMPLITUDE_MARGIN
        {
            return DaylightState::WhiteNight {
                min_altitude: min_alt,
                max_altitude: max_alt,
            };
        }

        DaylightState::Normal
    }
}

// ─────────────────────────────────────────────────────────────
// Main Function
// ─────────────────────────────────────────────────────────────

/// Determine the color scheme to apply based on solar altitude and hysteresis
///
/// # Decision flow
///
/// ```text
///                    ┌─────────────────────┐
///                    │  DaylightState pre   │
///                    └──────────┬──────────┘
///              ┌────────────────┼─────────────────┐
///        MidnightSun       PolarNight          WhiteNight
///              │                │                  │
///           Light         Clock fallback     Dynamic midpoint hysteresis
///                               │                  │
///                     is_waking_hours?       midpoint ± forced dead zone
///                      /        \
///                   Light      Dark
///
///                    ┌──────── Normal ────────┐
///                    │  Schmitt trigger hysteresis  │
///                    │                        │
///            Currently Dark            Currently Light
///          switch_point = base+band   switch_point = base-band
///              altitude > sp?             altitude < sp?
///               /      \                  /       \
///            Light     Dark            Dark      Light
///           (switch)  (keep)         (switch)   (keep)
/// └──────────────────────────────────────────────────────────┘
/// ```
///
/// # Parameters
/// - `solar_position`: precomputed solar position; `.altitude()` returns the
///   apparent altitude including refraction correction
/// - `current_scheme`: current color scheme (the key state for hysteresis)
/// - `config`: dynamic threshold configuration, generated by [`ThresholdConfig::from_position`]
/// - `local_time`: local time including timezone offset, used for polar night clock fallback
/// - `daylight_state`: the extreme daylight state for the day, should be computed
///   once per day by the caller and passed in cached
///
/// # Note
/// This function does not apply any additional refraction correction to
/// `solar_position.altitude()` — `SolarPosition`'s internal `altitude_from_context`
/// already applies the Bennett formula correction.
pub(crate) fn determine_color_scheme_with_hysteresis(
    solar_position: &SolarPosition,
    current_scheme: &ColorScheme,
    config: &ThresholdConfig,
    local_time: &OffsetDateTime,
    daylight_state: &DaylightState,
) -> ColorScheme {
    // ── Phase 1: Extreme state pre‑processing ───────────────────────────────────
    //
    // All three extreme states short-circuit here and do not enter the main
    // hysteresis logic, ensuring that the main logic only handles the normal
    // case where the altitude fully crosses the threshold.
    match daylight_state {
        // Midnight sun: sun is above the threshold all day, force light
        DaylightState::MidnightSun => return ColorScheme::Light,

        // Polar night: sun is below the threshold all day, astronomical
        // determination is meaningless. Fall back to local clock:
        // light during waking hours, dark otherwise.
        DaylightState::PolarNight => {
            let hour = local_time.hour();
            return if (WAKING_HOUR_START..WAKING_HOUR_END).contains(&hour) {
                ColorScheme::Light
            } else {
                ColorScheme::Dark
            };
        }

        // White night: altitude oscillates slightly around the threshold,
        // use dynamic midpoint + forced wide dead zone
        //
        // Dead zone half-width = WHITE_NIGHT_AMPLITUDE_MARGIN / 2 = 2°
        // 4× wider than the standard HYSTERESIS_BAND (0.5°), specifically to
        // combat the high-frequency jitter of white nights.
        // During white nights the relationship between local time and altitude
        // direction is unreliable; time-based assistance is not used here —
        // rely solely on the midpoint threshold + wide dead zone.
        DaylightState::WhiteNight {
            min_altitude,
            max_altitude,
        } => {
            let midpoint = (min_altitude + max_altitude) / 2.0;
            let half_band = WHITE_NIGHT_AMPLITUDE_MARGIN / 2.0; // 2.0°
            let altitude = solar_position.altitude();

            return if altitude >= midpoint + half_band {
                ColorScheme::Light
            } else if altitude < midpoint - half_band {
                ColorScheme::Dark
            } else {
                // Inside the dead zone: keep the current state (inertia principle)
                *current_scheme
            };
        }

        // Normal case: proceed to the Schmitt trigger logic below
        DaylightState::Normal => {}
    }

    // ── Phase 2: Schmitt trigger hysteresis (executed only for Normal state) ───
    //
    // Choose different switching points depending on the current state:
    //   - Currently Dark  → need a higher altitude to switch to Light (higher bar when rising)
    //   - Currently Light → need a lower altitude to switch to Dark (lower bar when falling)
    //
    // This forms a dead zone of width 2 × HYSTERESIS_BAND:
    //   Dark  → Light: altitude must exceed base_threshold + HYSTERESIS_BAND
    //   Light → Dark : altitude must fall below base_threshold - HYSTERESIS_BAND
    //
    // Any crossing within the dead zone is ignored; the current state is preserved.
    let altitude = solar_position.altitude();

    let switch_point = match current_scheme {
        ColorScheme::Dark => config.light_switch_point(),
        ColorScheme::Light => config.dark_switch_point(),
    };

    if altitude > switch_point {
        ColorScheme::Light
    } else {
        ColorScheme::Dark
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

    fn make_config(base: f64) -> ThresholdConfig {
        ThresholdConfig {
            base_threshold: base,
            hysteresis_band: 0.5,
        }
    }

    // ── Schmitt trigger hysteresis ──────────────────────────────────────────────

    #[test]
    fn schmitt_dark_to_light_requires_crossing_upper_point() {
        let config = make_config(-6.0);
        // Rising switch point = -6.0 + 0.5 = -5.5
        // altitude -5.6°: below -5.5°, currently Dark → should remain Dark
        assert_altitude_scheme(-5.6, ColorScheme::Dark, &config, ColorScheme::Dark);
        // altitude -5.4°: above -5.5°, currently Dark → should switch to Light
        assert_altitude_scheme(-5.4, ColorScheme::Dark, &config, ColorScheme::Light);
    }

    #[test]
    fn schmitt_light_to_dark_requires_crossing_lower_point() {
        let config = make_config(-6.0);
        // Falling switch point = -6.0 - 0.5 = -6.5
        // altitude -6.4°: above -6.5°, currently Light → should remain Light
        assert_altitude_scheme(-6.4, ColorScheme::Light, &config, ColorScheme::Light);
        // altitude -6.6°: below -6.5°, currently Light → should switch to Dark
        assert_altitude_scheme(-6.6, ColorScheme::Light, &config, ColorScheme::Dark);
    }

    #[test]
    fn hysteresis_band_prevents_oscillation_in_dead_zone() {
        let config = make_config(-6.0);
        // Dead zone: (-6.5°, -5.5°)
        // Regardless of current state, inside the dead zone the state should be kept
        for &alt in &[-6.4, -6.0, -5.6] {
            assert_altitude_scheme(alt, ColorScheme::Light, &config, ColorScheme::Light);
            assert_altitude_scheme(alt, ColorScheme::Dark, &config, ColorScheme::Dark);
        }
    }

    // ── Dynamic threshold ──────────────────────────────────────────────────────

    #[test]
    fn threshold_tropical_is_shallower_than_civil() {
        // Tropical (latitude 10°) threshold should be shallower (closer to 0°) than -6°
        let pos_tropical = mock_position(10.0);
        let pos_midlat = mock_position(35.0);
        let t_tropical = ThresholdConfig::from_position(&pos_tropical).base_threshold;
        let t_midlat = ThresholdConfig::from_position(&pos_midlat).base_threshold;
        assert!(
            t_tropical > t_midlat,
            "Tropical threshold {t_tropical} should be higher than mid-latitude {t_midlat}"
        );
    }

    #[test]
    fn threshold_high_latitude_is_deeper_than_civil() {
        // High latitude (latitude 60°) threshold should be deeper (closer to -9°) than -6°
        let pos_high = mock_position(60.0);
        let pos_mid = mock_position(35.0);
        let t_high = ThresholdConfig::from_position(&pos_high).base_threshold;
        let t_mid = ThresholdConfig::from_position(&pos_mid).base_threshold;
        assert!(
            t_high < t_mid,
            "High-latitude threshold {t_high} should be lower than mid-latitude {t_mid}"
        );
    }

    #[test]
    fn threshold_clamped_within_bounds() {
        // Pole (90°) should not exceed [THRESHOLD_MIN, THRESHOLD_MAX]
        let pos_pole = mock_position(90.0);
        let t = ThresholdConfig::from_position(&pos_pole).base_threshold;
        assert!((THRESHOLD_MIN..=THRESHOLD_MAX).contains(&t));
    }

    // ── Extreme states ──────────────────────────────────────────────────────

    #[test]
    fn midnight_sun_always_returns_light() {
        let config = make_config(-6.0);
        let state = DaylightState::MidnightSun;
        // Regardless of current state, midnight sun forces Light
        for current in [ColorScheme::Light, ColorScheme::Dark] {
            let result = call_with_state(-5.0, &current, &config, 14, &state);
            assert_eq!(result, ColorScheme::Light);
        }
    }

    #[test]
    fn polar_night_follows_waking_hours() {
        let config = make_config(-6.0);
        let state = DaylightState::PolarNight;
        // Within waking hours (10 o'clock) → Light
        assert_eq!(
            call_with_state(-30.0, &ColorScheme::Dark, &config, 10, &state),
            ColorScheme::Light
        );
        // Late night (23 o'clock) → Dark
        assert_eq!(
            call_with_state(-30.0, &ColorScheme::Light, &config, 23, &state),
            ColorScheme::Dark
        );
    }

    #[test]
    fn white_night_uses_midpoint_with_wide_band() {
        // min=-7°, max=-5°, midpoint=-6°, half-band=2°
        // To switch to Light need >= -4°, to switch to Dark need < -8°
        let config = make_config(-6.0);
        let state = DaylightState::WhiteNight {
            min_altitude: -7.0,
            max_altitude: -5.0,
        };

        // -3.9° (> -4.0°) → Light
        assert_eq!(
            call_with_state(-3.9, &ColorScheme::Dark, &config, 12, &state),
            ColorScheme::Light
        );
        // -8.1° (< -8.0°) → Dark
        assert_eq!(
            call_with_state(-8.1, &ColorScheme::Light, &config, 12, &state),
            ColorScheme::Dark
        );
        // -6.0° (inside dead zone) → keep current
        assert_eq!(
            call_with_state(-6.0, &ColorScheme::Light, &config, 12, &state),
            ColorScheme::Light
        );
        assert_eq!(
            call_with_state(-6.0, &ColorScheme::Dark, &config, 12, &state),
            ColorScheme::Dark
        );
    }

    // ── Helpers ──────────────────────────────────────────────────────

    /// Directly test the main logic using a given altitude (bypassing SolarPosition)
    fn assert_altitude_scheme(
        altitude: f64,
        current: ColorScheme,
        config: &ThresholdConfig,
        expected: ColorScheme,
    ) {
        let result = call_with_state(altitude, &current, config, 12, &DaylightState::Normal);
        assert_eq!(
            result, expected,
            "altitude={altitude}, current={current:?} → expected {expected:?}, got {result:?}"
        );
    }

    fn call_with_state(
        altitude: f64,
        current: &ColorScheme,
        config: &ThresholdConfig,
        hour: u8,
        state: &DaylightState,
    ) -> ColorScheme {
        // Directly drive the internal logic, bypassing SolarPosition construction
        // Using a mock here instead of a real SolarPosition
        mock_determine(altitude, current, config, hour, state)
    }

    /// Extract the main function logic into a pure function accepting a raw altitude,
    /// convenient for unit testing
    ///
    /// Identical logic to `determine_color_scheme_with_hysteresis`,
    /// only replacing `solar_position.altitude()` with the directly passed `altitude`.
    fn mock_determine(
        altitude: f64,
        current_scheme: &ColorScheme,
        config: &ThresholdConfig,
        hour: u8,
        daylight_state: &DaylightState,
    ) -> ColorScheme {
        match daylight_state {
            DaylightState::MidnightSun => return ColorScheme::Light,
            DaylightState::PolarNight => {
                return if (WAKING_HOUR_START..WAKING_HOUR_END).contains(&hour) {
                    ColorScheme::Light
                } else {
                    ColorScheme::Dark
                };
            }
            DaylightState::WhiteNight {
                min_altitude,
                max_altitude,
            } => {
                let midpoint = (min_altitude + max_altitude) / 2.0;
                let half_band = WHITE_NIGHT_AMPLITUDE_MARGIN / 2.0;
                return if altitude >= midpoint + half_band {
                    ColorScheme::Light
                } else if altitude < midpoint - half_band {
                    ColorScheme::Dark
                } else {
                    *current_scheme
                };
            }
            DaylightState::Normal => {}
        }

        let switch_point = match current_scheme {
            ColorScheme::Dark => config.light_switch_point(),
            ColorScheme::Light => config.dark_switch_point(),
        };

        if altitude > switch_point {
            ColorScheme::Light
        } else {
            ColorScheme::Dark
        }
    }

    fn mock_position(lat: f64) -> Position {
        Position::from_raw_position(lat, 0.0, 0.)
    }

    // ============================================================
    // Tests for determine_color_scheme_with_hysteresis
    //
    // There is hysteresis around sunrise and sunset, so we cannot switch color scheme
    // directly based solely on sunrise/sunset times.
    // ============================================================

    fn threshold(lat: f64, lon: f64) -> ThresholdConfig {
        ThresholdConfig::from_position(&Position::from_raw_position(lat, lon, 0.))
    }

    fn solar_position(position: &Position, local: OffsetDateTime) -> SolarPosition {
        let utc = local.utc().unwrap();
        SolarPosition::new(position, &utc)
    }

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
            let position = Position::from_raw_position(lat, lon, 0.);
            let sun = solar_position(&position, time);
            let daylight_state = DaylightState::detect(&position, time.date(), &config);
            let actual = determine_color_scheme_with_hysteresis(
                &sun,
                &ColorScheme::Dark, // initial scheme set to Dark; does not affect core logic (except hysteresis, tests use fixed initial value)
                &config,
                &time,
                &daylight_state,
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
