use std::fmt;

use serde::Deserialize;
use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Registry::{KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD},
    UI::WindowsAndMessaging::{HWND_BROADCAST, SendNotifyMessageW, WM_SETTINGCHANGE},
};

use crate::{
    domain::time::solar_transitions::{PolarState, SolarTransitions},
    error::DwallResult,
    infrastructure::platform::windows::registry_client::RegistryKey,
    utils::string::WideStringExt,
};

/// Represents the system color scheme (light or dark theme).
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    /// Returns the corresponding registry value: 1 for Light, 0 for Dark.
    fn as_u32(&self) -> u32 {
        match self {
            ColorScheme::Light => 1,
            ColorScheme::Dark => 0,
        }
    }

    /// Converts the scheme to little-endian bytes suitable for writing to the registry.
    fn as_le_bytes(&self) -> [u8; 4] {
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

/// Color scheme manager for Windows system theme management.
pub(crate) struct ColorSchemeManager;

impl ColorSchemeManager {
    const PERSONALIZE_KEY_PATH: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const APPS_THEME_VALUE: &str = "AppsUseLightTheme";
    const SYSTEM_THEME_VALUE: &str = "SystemUsesLightTheme";

    /// Retrieve the current system color scheme from registry.
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

    /// Set the system color scheme in the registry.
    pub(crate) fn set_color_scheme(scheme: &ColorScheme) -> DwallResult<()> {
        info!(scheme = %scheme, "Setting system color scheme");
        let registry_key = RegistryKey::open(Self::PERSONALIZE_KEY_PATH, KEY_SET_VALUE)?;

        let value = scheme.as_le_bytes();

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

/// Set the system color scheme, checking first if it needs to be changed.
pub(crate) fn set_color_scheme(color_scheme: ColorScheme) -> DwallResult<()> {
    let current_color_scheme = ColorSchemeManager::get_current_scheme()?;
    if current_color_scheme == color_scheme {
        info!(scheme = %color_scheme, "Color scheme is already set");
        return Ok(());
    }

    info!(from = %current_color_scheme, to = %color_scheme, "Changing color scheme");
    ColorSchemeManager::set_color_scheme(&color_scheme)?;

    if !verify_theme_change(&color_scheme)? {
        warn!("Theme change may not have been applied correctly");
    }

    Ok(())
}

/// Verify that the theme change was applied correctly by reading the registry
/// after a short delay.
fn verify_theme_change(expected: &ColorScheme) -> DwallResult<bool> {
    std::thread::sleep(std::time::Duration::from_millis(100));
    let actual = ColorSchemeManager::get_current_scheme()?;
    Ok(&actual == expected)
}

/// Notify the system about theme changes by broadcasting a settings change message.
fn notify_theme_change() -> DwallResult<()> {
    trace!("Broadcasting theme change notifications");

    // Build a null-terminated wide string for the notification parameter.
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

/// Schedule for switching between light and dark mode, similar to macOS appearance schedule.
#[derive(Debug)]
pub struct SwitchSchedule {
    /// Unix timestamp when the system should switch to Dark mode.
    to_dark_at: Option<u64>,
    /// Unix timestamp when the system should switch to Light mode.
    to_light_at: Option<u64>,
}

impl SwitchSchedule {
    /// Construct from solar transitions with configurable offsets.
    ///
    /// `sunset_offset_secs`: seconds after sunset to switch to Dark (0 = switch at sunset).
    /// `sunrise_offset_secs`: seconds before sunrise to switch to Light (negative = advance the switch time).
    pub fn from_transitions(
        transitions: &SolarTransitions,
        sunset_offset_secs: i16,
        sunrise_offset_secs: i16,
    ) -> Self {
        Self {
            to_dark_at: transitions
                .sunset
                .map(|t| (t.timestamp() as i64 + sunset_offset_secs as i64).max(0) as u64),
            to_light_at: transitions
                .sunrise
                .map(|t| (t.timestamp() as i64 + sunrise_offset_secs as i64).max(0) as u64),
        }
    }
}

/// Hysteresis duration (5 minutes) to avoid rapid toggling around transition boundaries.
const SCHEDULE_HYSTERESIS_SECS: u64 = 300; // +/- 5 minutes

/// Determine the appropriate color scheme based on the given schedule and polar state.
///
/// For polar day/night, the choice is immediate; for normal conditions the function
/// respects the configured switching times and adds a hysteresis period to prevent
/// frequent changes near the boundary.
pub(crate) fn determine_color_scheme_by_schedule(
    now_timestamp: u64,
    schedule: &SwitchSchedule,
    polar_state: PolarState,
) -> ColorScheme {
    match polar_state {
        PolarState::PolarDay => return ColorScheme::Light,
        PolarState::PolarNight => return ColorScheme::Dark,
        PolarState::Normal => {}
    }

    match (schedule.to_light_at, schedule.to_dark_at) {
        (Some(light), Some(dark)) => {
            // Use the most recent transition point that has passed.
            if light > dark {
                // Sunrise after sunset is unusual; defensively return Light.
                ColorScheme::Light
            } else if now_timestamp >= dark + SCHEDULE_HYSTERESIS_SECS {
                ColorScheme::Dark
            } else if now_timestamp >= light + SCHEDULE_HYSTERESIS_SECS {
                ColorScheme::Light
            } else {
                // Before sunrise, default to dark.
                ColorScheme::Dark
            }
        }
        (None, Some(_)) => ColorScheme::Dark, // No sunrise = polar night (handled above).
        (Some(_), None) => ColorScheme::Light, // No sunset = polar day.
        (None, None) => ColorScheme::Dark,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::time::solar_transitions::SolarTransitions;
    use crate::utils::datetime::UtcDateTime;

    /// Helper to create a SolarTransitions from optional Unix timestamps.
    fn solar_transitions(sunrise: Option<u64>, sunset: Option<u64>) -> SolarTransitions {
        SolarTransitions {
            sunrise: sunrise.map(UtcDateTime::from_timestamp),
            sunset: sunset.map(UtcDateTime::from_timestamp),
            solar_noon: UtcDateTime::from_timestamp(0), // arbitrary, not used by from_transitions
            polar_state: PolarState::Normal,
        }
    }

    /// Helper to create a SwitchSchedule with given timestamps.
    fn schedule(light: Option<u64>, dark: Option<u64>) -> SwitchSchedule {
        SwitchSchedule {
            to_light_at: light,
            to_dark_at: dark,
        }
    }

    #[test]
    fn test_color_scheme_as_u32() {
        assert_eq!(ColorScheme::Light.as_u32(), 1);
        assert_eq!(ColorScheme::Dark.as_u32(), 0);
    }

    #[test]
    fn test_color_scheme_display() {
        assert_eq!(format!("{}", ColorScheme::Light), "Light");
        assert_eq!(format!("{}", ColorScheme::Dark), "Dark");
    }

    #[test]
    fn test_color_scheme_to_le_bytes() {
        assert_eq!(ColorScheme::Light.as_le_bytes(), 1u32.to_le_bytes());
        assert_eq!(ColorScheme::Dark.as_le_bytes(), 0u32.to_le_bytes());
    }

    #[test]
    fn test_switch_schedule_from_transitions_no_offset() {
        let t = solar_transitions(Some(1_620_000_000), Some(1_620_060_000));
        let schedule = SwitchSchedule::from_transitions(&t, 0, 0);
        assert_eq!(schedule.to_light_at, Some(1_620_000_000));
        assert_eq!(schedule.to_dark_at, Some(1_620_060_000));
    }

    #[test]
    fn test_switch_schedule_from_transitions_with_offsets() {
        let t = solar_transitions(Some(1000), Some(2000));
        let schedule = SwitchSchedule::from_transitions(&t, 300, -300);
        assert_eq!(schedule.to_dark_at, Some(2300)); // 2000 + 300
        assert_eq!(schedule.to_light_at, Some(700)); // 1000 - 300
    }

    #[test]
    fn test_switch_schedule_from_transitions_offset_clamp_to_zero() {
        let t = solar_transitions(Some(10), Some(20));
        let schedule = SwitchSchedule::from_transitions(&t, 0, -100);
        assert_eq!(schedule.to_light_at, Some(0)); // clamped to 0
        assert_eq!(schedule.to_dark_at, Some(20));
    }

    #[test]
    fn test_polar_day_returns_light() {
        let sched = schedule(Some(100), Some(200));
        assert_eq!(
            determine_color_scheme_by_schedule(150, &sched, PolarState::PolarDay),
            ColorScheme::Light
        );
    }

    #[test]
    fn test_polar_night_returns_dark() {
        let sched = schedule(Some(100), Some(200));
        assert_eq!(
            determine_color_scheme_by_schedule(150, &sched, PolarState::PolarNight),
            ColorScheme::Dark
        );
    }

    #[test]
    fn test_normal_both_some_before_light_hysteresis() {
        // Hysteresis adds 300s to both timestamps.
        // now=150 is before light+300 (400) and dark is 200+300=500 -> default Dark.
        assert_eq!(
            determine_color_scheme_by_schedule(
                150,
                &schedule(Some(100), Some(200)),
                PolarState::Normal
            ),
            ColorScheme::Dark
        );
    }

    #[test]
    fn test_normal_both_some_after_light_hysteresis() {
        // now=450: >=400 but <500 -> Light.
        assert_eq!(
            determine_color_scheme_by_schedule(
                450,
                &schedule(Some(100), Some(200)),
                PolarState::Normal
            ),
            ColorScheme::Light
        );
    }

    #[test]
    fn test_normal_both_some_after_dark_hysteresis() {
        // now=550: >=500 -> Dark.
        assert_eq!(
            determine_color_scheme_by_schedule(
                550,
                &schedule(Some(100), Some(200)),
                PolarState::Normal
            ),
            ColorScheme::Dark
        );
    }

    #[test]
    fn test_normal_only_dark_returns_dark() {
        assert_eq!(
            determine_color_scheme_by_schedule(0, &schedule(None, Some(200)), PolarState::Normal),
            ColorScheme::Dark
        );
    }

    #[test]
    fn test_normal_only_light_returns_light() {
        assert_eq!(
            determine_color_scheme_by_schedule(0, &schedule(Some(100), None), PolarState::Normal),
            ColorScheme::Light
        );
    }

    #[test]
    fn test_normal_none_returns_dark() {
        assert_eq!(
            determine_color_scheme_by_schedule(0, &schedule(None, None), PolarState::Normal),
            ColorScheme::Dark
        );
    }

    #[test]
    fn test_normal_defense_light_greater_than_dark() {
        // Defensive: if light > dark, return Light.
        assert_eq!(
            determine_color_scheme_by_schedule(
                0,
                &schedule(Some(300), Some(200)),
                PolarState::Normal
            ),
            ColorScheme::Light
        );
    }
}
