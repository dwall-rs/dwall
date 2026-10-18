//! Automatic light/dark color scheme switching based on solar position.

use std::fmt;
use std::time::Duration;

use serde::Deserialize;
use time::{Date, OffsetDateTime};

use crate::config::Config;
use crate::error::DwallResult;
use crate::solar::{Position, SolarPosition};
use crate::utils::get_cache;

// ── Color scheme ────────────────────────────────────────────────────────────

/// Trait for managing system color scheme
///
/// Implemented by the platform backend (Windows registry + broadcast).
pub trait ColorSchemeProvider {
    /// Retrieves the current system color scheme
    fn get_current_scheme(&self) -> DwallResult<ColorScheme>;

    /// Sets the system color scheme
    fn set_color_scheme(&self, scheme: ColorScheme) -> DwallResult<()>;
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
    pub(crate) const fn to_le_bytes(self) -> [u8; 4] {
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

// ── Astronomical constants ──────────────────────────────────────────────────

/// Civil twilight threshold (degrees). When the center of the sun is 6° below
/// the horizon the scattered light has largely disappeared; this is the
/// baseline used by macOS/GNOME for automatic switching.
const CIVIL_TWILIGHT_DEG: f64 = -6.0;

/// Hysteresis half-bandwidth (degrees): the switching line is
/// `base_threshold ± HYSTERESIS_BAND`, giving a 1° dead zone (~10 min of sun
/// travel around sunrise/sunset).
const HYSTERESIS_BAND: f64 = 0.5;

/// White-night amplitude trigger threshold (degrees).
const WHITE_NIGHT_AMPLITUDE_MARGIN: f64 = 4.0;

/// Polar-night clock fallback: waking hours start hour (local, inclusive)
const WAKING_HOUR_START: u8 = 7;

/// Polar-night clock fallback: waking hours end hour (local, exclusive)
const WAKING_HOUR_END: u8 = 18;

/// Arctic / Antarctic Circle latitude (degrees)
const ARCTIC_CIRCLE_LAT: f64 = 66.5;

/// High-latitude boundary (degrees)
const HIGH_LAT_BOUNDARY: f64 = 45.0;

/// Tropic latitude (degrees)
const TROPIC_LAT: f64 = 23.5;

/// Polar base threshold (degrees): adjusted deeper from this value
const POLAR_BASE_THRESHOLD: f64 = -8.0;

/// Maximum deepening inside polar circles (degrees)
const POLAR_MAX_ADJUSTMENT: f64 = 4.0;

/// Maximum deepening at high latitudes (degrees)
const HIGH_LAT_MAX_ADJUSTMENT: f64 = 3.0;

/// Maximum shallowing in the tropics (degrees)
const TROPICAL_MAX_ADJUSTMENT: f64 = 1.5;

/// Lower clamp for the threshold (degrees)
const THRESHOLD_MIN: f64 = -12.0;

/// Upper clamp for the threshold (degrees)
const THRESHOLD_MAX: f64 = -4.5;

// ── Dynamic threshold ───────────────────────────────────────────────────────

/// Dynamic switching threshold based on geographic location.
///
/// Twilight duration varies significantly across latitudes:
/// - **Tropics** (< 23.5°): the sun crosses the horizon nearly vertically,
///   twilight is very short. The threshold is raised toward -4.5°.
/// - **Mid-latitudes** (23.5°–45°): standard civil twilight threshold of -6°.
/// - **High latitudes** (45°–66.5°): the sun crosses at a shallow angle; the
///   threshold is deepened toward -9°.
/// - **Inside the polar circles** (> 66.5°): deepest threshold (-8° to -12°).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThresholdConfig {
    pub(crate) base_threshold: f64,
    pub(crate) hysteresis_band: f64,
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

    /// Compute the base threshold by latitude zone.
    ///
    /// Uses `cos(lat)` as the latitude factor: 1.0 at the equator, 0.0 at the
    /// poles, which reflects how the sun's path angle relative to the horizon
    /// varies with latitude.
    fn calculate_threshold(position: &Position) -> f64 {
        let abs_lat = position.latitude().abs();
        let lat_rad = abs_lat.to_radians();
        let latitude_factor = lat_rad.cos();

        let threshold = if abs_lat > ARCTIC_CIRCLE_LAT {
            let extreme_factor = (abs_lat - ARCTIC_CIRCLE_LAT) / TROPIC_LAT;
            POLAR_BASE_THRESHOLD - extreme_factor * POLAR_MAX_ADJUSTMENT
        } else if abs_lat > HIGH_LAT_BOUNDARY {
            let adjustment = (1.0 - latitude_factor) * HIGH_LAT_MAX_ADJUSTMENT;
            CIVIL_TWILIGHT_DEG - adjustment
        } else if abs_lat < TROPIC_LAT {
            let tropical_factor = (TROPIC_LAT - abs_lat) / TROPIC_LAT;
            CIVIL_TWILIGHT_DEG + tropical_factor * TROPICAL_MAX_ADJUSTMENT
        } else {
            CIVIL_TWILIGHT_DEG
        };

        threshold.clamp(THRESHOLD_MIN, THRESHOLD_MAX)
    }

    /// Lower altitude bound for switching to Light
    #[inline]
    pub(crate) fn light_switch_point(&self) -> f64 {
        self.base_threshold + self.hysteresis_band
    }

    /// Upper altitude bound for switching to Dark
    #[inline]
    pub(crate) fn dark_switch_point(&self) -> f64 {
        self.base_threshold - self.hysteresis_band
    }
}

// ── Extreme daylight state ──────────────────────────────────────────────────

/// The extreme daylight state for a given day.
///
/// Determined by sampling [`SolarPosition::altitude()`] (which already includes
/// refraction correction) at 24 whole hours of the day. Should be updated once
/// a day and cached.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DaylightState {
    /// Normal day/night cycle, using standard hysteresis logic
    Normal,

    /// Midnight sun: the lowest altitude of the day is still above the base
    /// threshold. Force light mode.
    MidnightSun,

    /// Polar night: the highest altitude of the day is still below the base
    /// threshold. Fall back to local clock waking-hour range judgement.
    PolarNight,

    /// White night: the sun oscillates near the base threshold without fully
    /// crossing the hysteresis dead zone.
    WhiteNight {
        /// Minimum apparent altitude of the day (degrees, including refraction)
        min_altitude: f64,
        /// Maximum apparent altitude of the day (degrees, including refraction)
        max_altitude: f64,
    },
}

impl DaylightState {
    /// Detect the extreme state by sampling solar altitudes at 24 whole hours.
    pub fn detect(position: &Position, date: Date, config: &ThresholdConfig) -> Self {
        let altitudes: Vec<f64> = (0u8..24)
            .filter_map(|h| {
                date.with_hms(h, 0, 0)
                    .ok()
                    .map(|dt| SolarPosition::new(position, &dt.assume_utc()).altitude())
            })
            .collect();

        if altitudes.is_empty() {
            return DaylightState::Normal;
        }

        let min_alt = altitudes.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_alt = altitudes.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        if min_alt >= config.base_threshold {
            return DaylightState::MidnightSun;
        }

        if max_alt < config.base_threshold {
            return DaylightState::PolarNight;
        }

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

// ── Decision engine ─────────────────────────────────────────────────────────

/// Determine the color scheme to apply based on solar altitude and hysteresis.
///
/// Handles three extreme states (midnight sun, polar night, white night) before
/// falling back to a Schmitt-trigger hysteresis in the normal case. This
/// function does not apply any additional refraction correction:
/// `solar_position.altitude()` already includes it.
pub(crate) fn determine_color_scheme_with_hysteresis(
    solar_position: &SolarPosition,
    current_scheme: &ColorScheme,
    config: &ThresholdConfig,
    local_time: &OffsetDateTime,
    daylight_state: &DaylightState,
) -> ColorScheme {
    match daylight_state {
        DaylightState::MidnightSun => return ColorScheme::Light,

        DaylightState::PolarNight => {
            let hour = local_time.hour();
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
            let altitude = solar_position.altitude();

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

    // Schmitt trigger hysteresis:
    //   Dark  → Light: altitude must exceed base_threshold + HYSTERESIS_BAND
    //   Light → Dark : altitude must fall below base_threshold - HYSTERESIS_BAND
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

// ── Applier ─────────────────────────────────────────────────────────────────

/// Applies the system color scheme based on solar position.
pub(crate) struct ColorSchemeApplier<T: ColorSchemeProvider> {
    color_scheme_provider: T,
}

impl<T: ColorSchemeProvider> ColorSchemeApplier<T> {
    pub(crate) fn new(color_scheme_provider: T) -> Self {
        Self {
            color_scheme_provider,
        }
    }
}

// ── Cached derivations ──────────────────────────────────────────────────────

/// The threshold configuration together with the position it was computed from.
///
/// Both this and [`CachedDaylightState`] exist because the cache in
/// [`crate::utils`] is keyed by *type* alone. Storing only the value meant a cached
/// derivation stayed valid for its whole TTL regardless of its inputs: the threshold
/// survived the user moving (or editing their manual coordinates), and the daylight
/// state detected for one day was still served on the next one, where the sun's
/// path — and so the state — has moved on.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CachedThreshold {
    pub(crate) position: Position,
    pub(crate) config: ThresholdConfig,
}

/// The daylight state together with the inputs it was detected from.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CachedDaylightState {
    pub(crate) position: Position,
    pub(crate) date: Date,
    pub(crate) state: DaylightState,
}

/// How long a derived value may stay cached before it is recomputed.
///
/// The inputs are checked on every read, so this is only a ceiling for a value whose
/// inputs never change.
const DERIVED_TTL: Duration = Duration::from_hours(24);

/// How many times a value was really recomputed on this thread.
///
/// Thread-local, like the cache it measures, so a test observes its own work.
#[cfg(test)]
mod counters {
    use std::cell::Cell;

    thread_local! {
        pub(super) static THRESHOLDS: Cell<u32> = const { Cell::new(0) };
        pub(super) static DAYLIGHT_STATES: Cell<u32> = const { Cell::new(0) };
    }

    pub(super) fn bump(counter: &'static std::thread::LocalKey<Cell<u32>>) {
        counter.with(|value| value.set(value.get() + 1));
    }

    pub(super) fn read(counter: &'static std::thread::LocalKey<Cell<u32>>) -> u32 {
        counter.with(Cell::get)
    }
}

/// The threshold for `position`, recomputed when the position changes.
fn cached_threshold(position: &Position) -> ThresholdConfig {
    let cache = get_cache();

    if let Some(cached) = cache.get::<CachedThreshold>()
        && cached.position == *position
    {
        return cached.config;
    }

    let config = ThresholdConfig::from_position(position);
    cache.set(
        CachedThreshold {
            position: *position,
            config,
        },
        DERIVED_TTL,
    );

    #[cfg(test)]
    counters::bump(&counters::THRESHOLDS);

    config
}

/// The daylight state for `position` on `date`, recomputed when either changes.
fn cached_daylight_state(
    position: &Position,
    date: Date,
    threshold: &ThresholdConfig,
) -> DaylightState {
    let cache = get_cache();

    if let Some(cached) = cache.get::<CachedDaylightState>()
        && cached.position == *position
        && cached.date == date
    {
        return cached.state;
    }

    let state = DaylightState::detect(position, date, threshold);
    cache.set(
        CachedDaylightState {
            position: *position,
            date,
            state,
        },
        DERIVED_TTL,
    );

    #[cfg(test)]
    counters::bump(&counters::DAYLIGHT_STATES);

    state
}

impl<T: ColorSchemeProvider> ColorSchemeApplier<T> {
    /// The scheme the applier would choose at `now`, without applying anything.
    ///
    /// Split out of [`update_color_scheme`](Self::update_color_scheme) so the schedule
    /// can ask the same question about a future instant. A schedule that decided
    /// differently from the cycle would wake at the wrong times, which is worse than
    /// having no schedule at all.
    pub(crate) fn scheme_at(
        &self,
        now: &OffsetDateTime,
        geographic_position: &Position,
        solar_position: &SolarPosition,
        current_scheme: ColorScheme,
    ) -> ColorScheme {
        let threshold_config = cached_threshold(geographic_position);
        let daylight_state =
            cached_daylight_state(geographic_position, now.date(), &threshold_config);

        determine_color_scheme_with_hysteresis(
            solar_position,
            &current_scheme,
            &threshold_config,
            now,
            &daylight_state,
        )
    }

    /// The scheme the system is showing now, as far as this applier can tell.
    pub(crate) fn current_scheme(&self) -> DwallResult<ColorScheme> {
        self.color_scheme_provider.get_current_scheme()
    }

    /// Updates the system color scheme based on the current solar position.
    pub(crate) fn update_color_scheme(
        &self,
        config: &Config,
        now: &OffsetDateTime,
        geographic_position: &Position,
        solar_position: &SolarPosition,
    ) -> DwallResult<()> {
        if !config.auto_detect_color_scheme() {
            return Ok(());
        }

        let current_color_scheme = self.color_scheme_provider.get_current_scheme()?;
        let solar_based_color_scheme = self.scheme_at(
            now,
            geographic_position,
            solar_position,
            current_color_scheme,
        );

        debug!(
            color_scheme = ?solar_based_color_scheme,
            sun_altitude = solar_position.altitude(),
            "Automatically updating system color scheme based on solar position"
        );

        if let Err(color_scheme_error) = self
            .color_scheme_provider
            .set_color_scheme(solar_based_color_scheme)
        {
            warn!(
                error = %color_scheme_error,
                "Failed to update system color scheme, continuing with other operations"
            );
        }

        Ok(())
    }
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

    // ── ColorScheme basics ──────────────────────────────────────────────────

    // ── Derived-value caches ────────────────────────────────────────────────
    //
    // The cache in `utils` is keyed by type, so these check what the cached
    // derivation is allowed to be reused for. Each test reads its thread-local
    // counter before and after, because the cache itself is thread-local.

    fn date(year: u16, month: Month, day: u8) -> Date {
        Date::new(year, month, day)
    }

    #[test]
    fn the_threshold_is_reused_while_the_position_is_unchanged() {
        let position = mock_position(35.0);
        let before = counters::read(&counters::THRESHOLDS);

        let first = cached_threshold(&position);
        let second = cached_threshold(&position);

        assert_eq!(first.base_threshold, second.base_threshold);
        assert_eq!(
            counters::read(&counters::THRESHOLDS) - before,
            1,
            "the second lookup has to come from the cache"
        );
    }

    /// The regression test for a moved user: the cached threshold stayed valid for
    /// its whole 24-hour TTL, so the switching threshold kept describing the old
    /// latitude.
    #[test]
    fn the_threshold_cache_is_dropped_when_the_position_changes() {
        let before = counters::read(&counters::THRESHOLDS);

        let tropics = cached_threshold(&mock_position(5.0));
        let polar = cached_threshold(&mock_position(70.0));

        assert_ne!(
            tropics.base_threshold, polar.base_threshold,
            "the two latitudes need different thresholds"
        );
        assert_eq!(
            polar.base_threshold,
            ThresholdConfig::from_position(&mock_position(70.0)).base_threshold
        );
        assert_eq!(
            counters::read(&counters::THRESHOLDS) - before,
            2,
            "a new position has to be recomputed"
        );
    }

    #[test]
    fn the_daylight_state_is_reused_for_the_same_day_and_position() {
        let position = mock_position(35.0);
        let threshold = cached_threshold(&position);
        let today = date(2026, Month::June, 21);
        let before = counters::read(&counters::DAYLIGHT_STATES);

        let first = cached_daylight_state(&position, today, &threshold);
        let second = cached_daylight_state(&position, today, &threshold);

        assert_eq!(first, second);
        assert_eq!(
            counters::read(&counters::DAYLIGHT_STATES) - before,
            1,
            "the same day must not be detected twice"
        );
    }

    /// The regression test for the day rolling over: the state detected yesterday
    /// stayed cached for up to 24 more hours, although the sun's path — and so the
    /// extreme state — belongs to the date it was computed for.
    #[test]
    fn daylight_state_is_recomputed_on_a_new_day() {
        let position = mock_position(66.0);
        let threshold = cached_threshold(&position);
        let before = counters::read(&counters::DAYLIGHT_STATES);

        cached_daylight_state(&position, date(2026, Month::June, 21), &threshold);
        cached_daylight_state(&position, date(2026, Month::June, 22), &threshold);

        assert_eq!(
            counters::read(&counters::DAYLIGHT_STATES) - before,
            2,
            "a new date has to be detected again"
        );

        // And the cached entry now describes the new date.
        assert_eq!(
            get_cache().get::<CachedDaylightState>().unwrap().date,
            date(2026, Month::June, 22)
        );
    }

    #[test]
    fn the_daylight_state_is_recomputed_when_the_position_changes() {
        let threshold = ThresholdConfig::default_civil();
        let today = date(2026, Month::June, 21);
        let before = counters::read(&counters::DAYLIGHT_STATES);

        cached_daylight_state(&mock_position(35.0), today, &threshold);
        cached_daylight_state(&mock_position(78.0), today, &threshold);

        assert_eq!(
            counters::read(&counters::DAYLIGHT_STATES) - before,
            2,
            "another latitude is another day's sun path"
        );
    }

    // ── Latitude zone boundaries ────────────────────────────────────────────

    /// The three documented boundaries: 23.5 (tropics end), 45 (high latitudes
    /// start) and 66.5 (polar circles).
    #[test]
    fn threshold_at_the_documented_latitude_boundaries() {
        // Below the tropics the threshold is raised, at the boundary it is the plain
        // civil twilight value.
        assert!(
            ThresholdConfig::from_position(&mock_position(23.4)).base_threshold
                > CIVIL_TWILIGHT_DEG
        );
        assert_eq!(
            ThresholdConfig::from_position(&mock_position(TROPIC_LAT)).base_threshold,
            CIVIL_TWILIGHT_DEG
        );

        // 45 itself is still mid-latitude; just above it the threshold deepens.
        assert_eq!(
            ThresholdConfig::from_position(&mock_position(HIGH_LAT_BOUNDARY)).base_threshold,
            CIVIL_TWILIGHT_DEG
        );
        assert!(
            ThresholdConfig::from_position(&mock_position(45.1)).base_threshold
                < CIVIL_TWILIGHT_DEG
        );

        // 66.5 itself uses the high-latitude branch; just above it the polar branch
        // takes over and the threshold drops below it.
        let high = ThresholdConfig::from_position(&mock_position(ARCTIC_CIRCLE_LAT)).base_threshold;
        let polar = ThresholdConfig::from_position(&mock_position(66.6)).base_threshold;
        assert!(
            polar < high,
            "the polar branch is deeper: {polar} vs {high}"
        );
        assert!(polar >= THRESHOLD_MIN);
        assert!(high >= THRESHOLD_MIN);
    }

    #[test]
    fn the_threshold_never_leaves_the_documented_range() {
        for latitude in [
            -90.0, -70.0, -45.0, -23.5, 0.0, 23.5, 45.0, 66.5, 70.0, 90.0,
        ] {
            let threshold = ThresholdConfig::from_position(&mock_position(latitude)).base_threshold;

            assert!(
                (THRESHOLD_MIN..=THRESHOLD_MAX).contains(&threshold),
                "latitude {latitude} produced {threshold}"
            );
        }
    }

    /// The southern hemisphere is judged by the same rules; the sign of the latitude
    /// must not change the threshold.
    #[test]
    fn the_threshold_is_symmetric_about_the_equator() {
        for latitude in [10.0, 35.0, 50.0, 70.0] {
            assert_eq!(
                ThresholdConfig::from_position(&mock_position(latitude)).base_threshold,
                ThresholdConfig::from_position(&mock_position(-latitude)).base_threshold,
            );
        }
    }

    // ── Extreme daylight states at real locations ───────────────────────────

    /// Svalbard in June: the sun does not set, so the state is midnight sun at any
    /// hour and the scheme is forced to light.
    #[test]
    fn midnight_sun_at_svalbard_in_june() {
        let position = Position::from_raw_position(78.22, 15.65, 0.0);
        let threshold = ThresholdConfig::from_position(&position);
        let day = date(2026, Month::June, 21);

        let state = DaylightState::detect(&position, day, &threshold);

        assert_eq!(state, DaylightState::MidnightSun);
    }

    /// Alert in December: the sun stays far below the threshold all day.
    #[test]
    fn polar_night_at_alert_in_december() {
        let position = Position::from_raw_position(82.5, -62.35, 0.0);
        let threshold = ThresholdConfig::from_position(&position);
        let day = date(2026, Month::December, 21);

        let state = DaylightState::detect(&position, day, &threshold);

        assert_eq!(state, DaylightState::PolarNight);
    }

    /// The polar-night fallback is a waking-hours judgement: dark before 7, light
    /// between 7 and 18, dark again after.
    #[test]
    fn polar_night_falls_back_to_the_local_clock() {
        let config = make_config(-9.0);
        let position = Position::from_raw_position(82.5, -62.35, 0.0);

        for (hour, expected) in [
            (0, ColorScheme::Dark),
            (6, ColorScheme::Dark),
            (7, ColorScheme::Light),
            (12, ColorScheme::Light),
            (17, ColorScheme::Light),
            (18, ColorScheme::Dark),
            (23, ColorScheme::Dark),
        ] {
            let time = OffsetDateTime::new(
                2026,
                Month::December,
                21,
                hour,
                0,
                0,
                "+00:00".parse().unwrap(),
            )
            .unwrap();
            let sun = solar_position(&position, time);

            let actual = determine_color_scheme_with_hysteresis(
                &sun,
                &ColorScheme::Dark,
                &config,
                &time,
                &DaylightState::PolarNight,
            );

            assert_eq!(actual, expected, "hour {hour}");
        }
    }

    /// Mid-latitudes keep the plain day/night cycle: the states are not triggered.
    #[test]
    fn a_mid_latitude_day_is_normal() {
        let position = Position::from_raw_position(39.9, 116.4, 0.0);
        let threshold = ThresholdConfig::from_position(&position);

        for (month, day) in [
            (Month::March, 21),
            (Month::June, 21),
            (Month::September, 21),
            (Month::December, 21),
        ] {
            assert_eq!(
                DaylightState::detect(&position, date(2026, month, day), &threshold),
                DaylightState::Normal,
                "{month:?} {day}"
            );
        }
    }

    // ── ColorScheme basics ──────────────────────────────────────────────────

    #[test]
    fn color_scheme_display_light() {
        assert_eq!(ColorScheme::Light.to_string(), "Light");
    }

    #[test]
    fn color_scheme_display_dark() {
        assert_eq!(ColorScheme::Dark.to_string(), "Dark");
    }

    #[test]
    fn color_scheme_to_le_bytes_light() {
        assert_eq!(ColorScheme::Light.to_le_bytes(), [1, 0, 0, 0]);
    }

    #[test]
    fn color_scheme_to_le_bytes_dark() {
        assert_eq!(ColorScheme::Dark.to_le_bytes(), [0, 0, 0, 0]);
    }

    #[test]
    fn color_scheme_equality() {
        assert_eq!(ColorScheme::Light, ColorScheme::Light);
        assert_eq!(ColorScheme::Dark, ColorScheme::Dark);
        assert_ne!(ColorScheme::Light, ColorScheme::Dark);
    }

    // ── ThresholdConfig ─────────────────────────────────────────────────────

    fn mock_position(lat: f64) -> Position {
        Position::from_raw_position(lat, 0.0, 0.)
    }

    #[test]
    fn threshold_tropical_is_shallower_than_civil() {
        let t_tropical = ThresholdConfig::from_position(&mock_position(10.0)).base_threshold;
        let t_midlat = ThresholdConfig::from_position(&mock_position(35.0)).base_threshold;
        assert!(
            t_tropical > t_midlat,
            "Tropical threshold {t_tropical} should be higher than mid-latitude {t_midlat}"
        );
    }

    #[test]
    fn threshold_high_latitude_is_deeper_than_civil() {
        let t_high = ThresholdConfig::from_position(&mock_position(60.0)).base_threshold;
        let t_mid = ThresholdConfig::from_position(&mock_position(35.0)).base_threshold;
        assert!(
            t_high < t_mid,
            "High-latitude threshold {t_high} should be lower than mid-latitude {t_mid}"
        );
    }

    #[test]
    fn threshold_clamped_within_bounds() {
        let t = ThresholdConfig::from_position(&mock_position(90.0)).base_threshold;
        assert!((THRESHOLD_MIN..=THRESHOLD_MAX).contains(&t));
    }

    #[test]
    fn default_civil_threshold() {
        let config = ThresholdConfig::default_civil();
        assert_eq!(config.base_threshold, CIVIL_TWILIGHT_DEG);
        assert_eq!(config.hysteresis_band, HYSTERESIS_BAND);
    }

    #[test]
    fn switch_points_symmetric_around_base() {
        let config = make_config(-6.0);
        assert_eq!(config.light_switch_point(), -5.5);
        assert_eq!(config.dark_switch_point(), -6.5);
    }

    // ── DaylightState ───────────────────────────────────────────────────────

    #[test]
    fn white_night_variant_holds_altitudes() {
        let state = DaylightState::WhiteNight {
            min_altitude: -7.0,
            max_altitude: -5.0,
        };
        match state {
            DaylightState::WhiteNight {
                min_altitude,
                max_altitude,
            } => {
                assert_eq!(min_altitude, -7.0);
                assert_eq!(max_altitude, -5.0);
            }
            _ => panic!("Expected WhiteNight variant"),
        }
    }

    #[test]
    fn daylight_state_copy_semantics() {
        let state = DaylightState::Normal;
        let copied = state;
        assert_eq!(state, copied);
    }

    // ── Decision engine: Schmitt trigger ────────────────────────────────────

    #[test]
    fn schmitt_dark_to_light_requires_crossing_upper_point() {
        let config = make_config(-6.0);
        assert_altitude_scheme(-5.6, ColorScheme::Dark, &config, ColorScheme::Dark);
        assert_altitude_scheme(-5.4, ColorScheme::Dark, &config, ColorScheme::Light);
    }

    #[test]
    fn schmitt_light_to_dark_requires_crossing_lower_point() {
        let config = make_config(-6.0);
        assert_altitude_scheme(-6.4, ColorScheme::Light, &config, ColorScheme::Light);
        assert_altitude_scheme(-6.6, ColorScheme::Light, &config, ColorScheme::Dark);
    }

    #[test]
    fn hysteresis_band_prevents_oscillation_in_dead_zone() {
        let config = make_config(-6.0);
        for &alt in &[-6.4, -6.0, -5.6] {
            assert_altitude_scheme(alt, ColorScheme::Light, &config, ColorScheme::Light);
            assert_altitude_scheme(alt, ColorScheme::Dark, &config, ColorScheme::Dark);
        }
    }

    // ── Decision engine: extreme states ─────────────────────────────────────

    #[test]
    fn midnight_sun_always_returns_light() {
        let config = make_config(-6.0);
        let state = DaylightState::MidnightSun;
        for current in [ColorScheme::Light, ColorScheme::Dark] {
            let result = call_with_state(-5.0, &current, &config, 14, &state);
            assert_eq!(result, ColorScheme::Light);
        }
    }

    #[test]
    fn polar_night_follows_waking_hours() {
        let config = make_config(-6.0);
        let state = DaylightState::PolarNight;
        assert_eq!(
            call_with_state(-30.0, &ColorScheme::Dark, &config, 10, &state),
            ColorScheme::Light
        );
        assert_eq!(
            call_with_state(-30.0, &ColorScheme::Light, &config, 23, &state),
            ColorScheme::Dark
        );
    }

    #[test]
    fn white_night_uses_midpoint_with_wide_band() {
        let config = make_config(-6.0);
        let state = DaylightState::WhiteNight {
            min_altitude: -7.0,
            max_altitude: -5.0,
        };

        assert_eq!(
            call_with_state(-3.9, &ColorScheme::Dark, &config, 12, &state),
            ColorScheme::Light
        );
        assert_eq!(
            call_with_state(-8.1, &ColorScheme::Light, &config, 12, &state),
            ColorScheme::Dark
        );
        assert_eq!(
            call_with_state(-6.0, &ColorScheme::Light, &config, 12, &state),
            ColorScheme::Light
        );
        assert_eq!(
            call_with_state(-6.0, &ColorScheme::Dark, &config, 12, &state),
            ColorScheme::Dark
        );
    }

    // ── Decision helpers ────────────────────────────────────────────────────

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
        mock_determine(altitude, current, config, hour, state)
    }

    /// Same logic as `determine_color_scheme_with_hysteresis`, but taking a raw
    /// altitude so the decision can be tested without constructing a
    /// `SolarPosition`.
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

    // ── Geographic integration tests ────────────────────────────────────────

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
                &ColorScheme::Dark,
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
        test_location(
            44.3037058,
            80.9801647,
            2026,
            Month::May,
            15,
            "+08:00",
            &[
                ((6, 30, 0), ColorScheme::Dark),
                ((7, 15, 0), ColorScheme::Light),
                ((12, 0, 0), ColorScheme::Light),
                ((21, 0, 0), ColorScheme::Light),
                ((22, 40, 0), ColorScheme::Dark),
                ((3, 0, 0), ColorScheme::Dark),
            ],
        );
    }

    #[test]
    fn test_greenland_midday_night() {
        test_location(
            61.,
            -45.,
            2026,
            Month::January,
            1,
            "-01:00",
            &[
                ((10, 00, 0), ColorScheme::Dark),
                ((10, 45, 0), ColorScheme::Light),
                ((12, 0, 0), ColorScheme::Light),
                ((15, 0, 0), ColorScheme::Light),
                ((18, 10, 0), ColorScheme::Dark),
                ((23, 0, 0), ColorScheme::Dark),
            ],
        );
    }

    #[test]
    fn test_iceland_midday_night() {
        test_location(
            66.3617958,
            -22.4390253,
            2026,
            Month::March,
            1,
            "+00:00",
            &[
                ((7, 30, 0), ColorScheme::Dark),
                ((9, 00, 0), ColorScheme::Light),
                ((12, 0, 0), ColorScheme::Light),
                ((18, 30, 0), ColorScheme::Light),
                ((18, 50, 0), ColorScheme::Light),
                ((23, 0, 0), ColorScheme::Dark),
            ],
        );
    }

    #[test]
    fn test_paris_midday_night() {
        test_location(
            48.8728329,
            2.3281715,
            2026,
            Month::May,
            1,
            "+02:00",
            &[
                ((5, 40, 0), ColorScheme::Dark),
                ((6, 35, 0), ColorScheme::Light),
                ((12, 0, 0), ColorScheme::Light),
                ((21, 00, 0), ColorScheme::Light),
                ((21, 30, 0), ColorScheme::Light),
                ((23, 0, 0), ColorScheme::Dark),
            ],
        );
    }

    #[test]
    fn test_kenya_midday_night() {
        test_location(
            0.3488742,
            40.1127608,
            2026,
            Month::December,
            1,
            "+03:00",
            &[
                ((5, 20, 0), ColorScheme::Dark),
                ((6, 3, 0), ColorScheme::Light),
                ((12, 0, 0), ColorScheme::Light),
                ((18, 10, 0), ColorScheme::Light),
                ((19, 00, 0), ColorScheme::Dark),
                ((23, 0, 0), ColorScheme::Dark),
            ],
        );
    }
}
