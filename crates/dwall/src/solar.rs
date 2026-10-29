//! Geographic position and solar position computation.
//!
//! Contains the pure solar math ([`SolarPosition`], [`SolarAngle`]) plus the
//! [`PositionProvider`] abstraction used to obtain the observer location.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use time::UtcDateTime;

use crate::config::PositionSource;
use crate::error::DwallResult;
use crate::utils::get_cache;

// ── Geographic position ─────────────────────────────────────────────────────

/// Error type for geolocation access operations
///
/// The messages are the whole diagnosis a user gets: they are written to the log, shown
/// as the engine's last error in the settings window, and printed by the command line,
/// so each one names the setting that fixes it.
#[derive(Debug, thiserror::Error)]
pub enum GeolocationAccessError {
    #[error(
        "Location access was denied for this app. Allow desktop apps in Settings → Privacy & security → Location, or set [position_source] type = \"MANUAL\" with fixed coordinates"
    )]
    Denied,
    #[error(
        "The Windows location service is off. Turn it on in Settings → Privacy & security → Location, or set [position_source] type = \"MANUAL\" with fixed coordinates"
    )]
    Unspecified,
}

/// Error type for coordinate-related operations
#[derive(Debug, thiserror::Error)]
pub enum CoordinateError {
    #[error("Invalid latitude: {0}. Must be between -90 and 90 degrees")]
    InvalidLatitude(f64),

    #[error("Invalid longitude: {0}. Must be between -180 and 180 degrees")]
    InvalidLongitude(f64),

    #[error("Invalid coordinates: latitude {0}, longitude {1}")]
    InvalidCoordinates(f64, f64),
}

/// Geographic position with latitude, longitude and altitude
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    latitude: f64,
    longitude: f64,
    altitude: f64,
}

impl Position {
    /// Creates a new Position with the given latitude, longitude and altitude
    pub fn new(latitude: f64, longitude: f64, altitude: f64) -> DwallResult<Self> {
        if !Self::is_valid_latitude(latitude) {
            return Err(CoordinateError::InvalidLatitude(latitude).into());
        }
        if !Self::is_valid_longitude(longitude) {
            return Err(CoordinateError::InvalidLongitude(longitude).into());
        }

        Ok(Position {
            latitude,
            longitude,
            altitude,
        })
    }

    /// Creates a new Position without validation.
    ///
    /// Only use when coordinates are guaranteed to be valid (e.g., from a
    /// trusted source).
    pub(crate) fn from_raw_position(latitude: f64, longitude: f64, altitude: f64) -> Self {
        Position {
            latitude,
            longitude,
            altitude,
        }
    }

    /// Checks if the given latitude is valid (between -90 and 90 degrees)
    pub fn is_valid_latitude(latitude: f64) -> bool {
        (-90.0..=90.0).contains(&latitude)
    }

    /// Checks if the given longitude is valid (between -180 and 180 degrees)
    pub fn is_valid_longitude(longitude: f64) -> bool {
        (-180.0..=180.0).contains(&longitude)
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn altitude(&self) -> f64 {
        self.altitude
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Position(lat: {}, lng: {}, alt: {})",
            self.latitude, self.longitude, self.altitude
        )
    }
}

/// Trait for providing geographic position data.
///
/// Implemented by the platform geolocation backend (Windows Geolocator).
pub trait PositionProvider {
    /// Retrieves the current geographic position
    fn get_current_position(&self) -> DwallResult<Position>;

    /// Checks if the application has permission to access location
    fn check_location_permission(&self) -> DwallResult<()>;
}

/// Resolves the current position from configuration, applying caching.
///
/// Automatic mode caches the position for `cache_minutes`; manual mode simply
/// returns the configured coordinates.
pub struct GeographicPositionProvider<'a, P: PositionProvider> {
    coordinate_source: &'a PositionSource,
    position_provider: P,
}

impl<'a, P: PositionProvider> GeographicPositionProvider<'a, P> {
    pub fn new(coordinate_source: &'a PositionSource, position_provider: P) -> Self {
        Self {
            coordinate_source,
            position_provider,
        }
    }

    fn get_fresh_position(&self) -> DwallResult<Position> {
        debug!("Using fresh geolocation data");
        self.position_provider.get_current_position()
    }

    fn get_manual_position(
        &self,
        latitude: f64,
        longitude: f64,
        altitude: f64,
    ) -> DwallResult<Position> {
        debug!(
            latitude = latitude,
            longitude = longitude,
            altitude = altitude,
            "Using manual position"
        );
        Position::new(latitude, longitude, altitude)
    }

    fn get_cached_position(&self, ttl: Duration) -> DwallResult<Position> {
        let cache = get_cache();
        match cache.get::<Position>() {
            Some(pos) => Ok(pos),
            None => {
                let pos = self.get_fresh_position()?;
                cache.set(pos, ttl);
                Ok(pos)
            }
        }
    }

    /// Retrieves the current position based on the configured coordinate source
    pub fn get_current_position(&self) -> DwallResult<Position> {
        match &self.coordinate_source {
            PositionSource::Automatic {
                update_on_each_calculation,
                cache_minutes: position_cache_minutes,
            } => {
                if *update_on_each_calculation {
                    self.get_fresh_position()
                } else {
                    self.get_cached_position(position_cache_ttl(*position_cache_minutes))
                }
            }
            PositionSource::Manual {
                latitude,
                longitude,
                altitude,
            } => self.get_manual_position(*latitude, *longitude, *altitude),
        }
    }
}

/// The cache lifetime for `cache_minutes`, bounded.
///
/// `Config::validate` rejects an out-of-range value, but this also has to survive a
/// configuration that reached the cache without being validated (the settings app
/// writes the file the daemon reads): `60 * minutes` used to overflow, and the
/// resulting duration panicked the addition inside the cache.
fn position_cache_ttl(cache_minutes: u64) -> Duration {
    /// A year, which no configuration has a reason to exceed and which `Instant`
    /// arithmetic can still represent.
    const MAX_POSITION_CACHE_TTL: Duration = Duration::from_secs(24 * 365 * 3600);

    Duration::from_secs(60u64.saturating_mul(cache_minutes)).min(MAX_POSITION_CACHE_TTL)
}

impl<P: PositionProvider> PositionProvider for GeographicPositionProvider<'_, P> {
    fn get_current_position(&self) -> DwallResult<Position> {
        Self::get_current_position(self)
    }

    /// Resolving a position is what this type does, so it asks the backend it wraps.
    fn check_location_permission(&self) -> DwallResult<()> {
        self.position_provider.check_location_permission()
    }
}

// ── Solar position ──────────────────────────────────────────────────────────

mod constants {
    pub(super) const EPOCH_J2000: f64 = 2451545.0;
    pub(super) const JULIAN_CENTURY_DAYS: f64 = 36525.0;
    pub(super) const EARTH_ROTATION_RATE: f64 = 15.0;
    pub(super) const HOURS_PER_DAY: f64 = 24.0;
    pub(super) const MINUTES_PER_HOUR: f64 = 60.0;
    pub(super) const SECONDS_PER_HOUR: f64 = 3600.0;
    pub(super) const MINUTES_PER_DAY: f64 = 1440.0;
    pub(super) const SECONDS_PER_DAY: f64 = 86400.0;
    pub(super) const ATMOSPHERIC_REFRACTION_MAX: f64 = 0.575;
}

/// Barometric pressure at `altitude_m`, relative to sea level.
///
/// Refraction depends on air pressure, and pressure falls with elevation. The
/// standard-atmosphere approximation is enough for a wallpaper match; a missing or
/// implausible reading falls back to sea level instead of poisoning the altitude.
fn pressure_ratio(altitude_m: f64) -> f64 {
    if !altitude_m.is_finite() {
        return 1.0;
    }

    let altitude_m = altitude_m.clamp(-500.0, 9_000.0);
    (1.0 - 2.255_77e-5 * altitude_m)
        .powf(5.255_88)
        .clamp(0.30, 1.10)
}

struct SolarContext {
    latitude_rad: f64,
    declination_rad: f64,
    hour_angle_rad: f64,
    /// Refraction scale from the observer's elevation (1.0 at sea level).
    pressure_ratio: f64,
}

impl SolarContext {
    fn new(position: &Position, date_time: &UtcDateTime) -> Self {
        let t = SolarCalc::julian_century_t(date_time);
        let epsilon_deg = SolarCalc::earth_axial_tilt(t);
        let declination_deg = SolarCalc::solar_declination(t, epsilon_deg);
        let eot_hours = SolarCalc::solar_time_correction(t, epsilon_deg);
        let decimal_hours = SolarCalc::decimal_hours(date_time);
        let hour_angle_deg = SolarCalc::hour_angle(decimal_hours, position.longitude(), eot_hours);

        Self {
            latitude_rad: position.latitude().to_radians(),
            declination_rad: declination_deg.to_radians(),
            hour_angle_rad: hour_angle_deg.to_radians(),
            pressure_ratio: pressure_ratio(position.altitude()),
        }
    }
}

struct SolarCalc;

impl SolarCalc {
    fn julian_day(date_time: &UtcDateTime) -> f64 {
        let (year, month, day, hour, minute, second) = date_time.ymd_hms();

        let y = year as i32;
        let m = month as i32;
        let d = day as i32;

        let a = (14 - m) / 12;
        let y2 = y + 4800 - a;
        let m2 = m + 12 * a - 3;
        let jdn = d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045;

        jdn as f64 - 0.5
            + hour as f64 / constants::HOURS_PER_DAY
            + minute as f64 / constants::MINUTES_PER_DAY
            + second as f64 / constants::SECONDS_PER_DAY
    }

    fn julian_century_t(date_time: &UtcDateTime) -> f64 {
        let jd = Self::julian_day(date_time);
        (jd - constants::EPOCH_J2000) / constants::JULIAN_CENTURY_DAYS
    }

    #[inline]
    fn earth_axial_tilt(t: f64) -> f64 {
        23.43929111 - 0.0130042 * t - 0.00000164 * t.powi(2) + 0.000000503 * t.powi(3)
    }

    fn solar_ecliptic_longitude(t: f64) -> f64 {
        let l0 = 280.4664567 + 36000.7698278 * t + 0.0003032028 * t.powi(2);

        let l1 = 1.914602 - 0.004817 * t - 0.000014 * t.powi(2);
        let l2 = 0.019993 - 0.000101 * t;
        let l3 = 0.000289;

        let m = (357.5291092 + 35999.0502909 * t).to_radians();

        let lambda = l0 + l1 * m.sin() + l2 * (2.0 * m).sin() + l3 * (3.0 * m).sin();

        ((lambda % 360.0) + 360.0) % 360.0
    }

    fn solar_declination(t: f64, epsilon_deg: f64) -> f64 {
        let lambda = Self::solar_ecliptic_longitude(t);
        let sin_delta = epsilon_deg.to_radians().sin() * lambda.to_radians().sin();
        sin_delta.asin().to_degrees()
    }

    fn solar_time_correction(t: f64, epsilon_deg: f64) -> f64 {
        let m_deg = 357.52911 + 35999.05029 * t - 0.0001537 * t.powi(2);
        let m = m_deg.to_radians();

        let l0_deg = 280.46646 + 36000.76983 * t + 0.0003032 * t.powi(2);

        let epsilon = epsilon_deg.to_radians();

        let c = (1.914602 - 0.004817 * t - 0.000014 * t.powi(2)) * m.sin()
            + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
            + 0.000289 * (3.0 * m).sin();

        let lambda = (l0_deg + c).to_radians();

        let alpha = (lambda.sin() * epsilon.cos()).atan2(lambda.cos());

        let mut diff = (l0_deg - alpha.to_degrees() - 0.0057183) % 360.0;
        if diff > 180.0 {
            diff -= 360.0;
        } else if diff < -180.0 {
            diff += 360.0;
        }

        diff * 4.0 / constants::MINUTES_PER_HOUR
    }

    #[inline]
    fn decimal_hours(date_time: &UtcDateTime) -> f64 {
        date_time.hour() as f64
            + date_time.minute() as f64 / constants::MINUTES_PER_HOUR
            + date_time.second() as f64 / constants::SECONDS_PER_HOUR
    }

    fn hour_angle(decimal_hours: f64, longitude: f64, eot_hours: f64) -> f64 {
        let local_apparent_time =
            decimal_hours + (longitude / constants::EARTH_ROTATION_RATE) + eot_hours;

        let raw_angle = constants::EARTH_ROTATION_RATE * (local_apparent_time - 12.0);

        let ha = ((raw_angle % 360.0) + 360.0) % 360.0;

        if ha > 180.0 { ha - 360.0 } else { ha }
    }

    fn atmospheric_refraction(apparent_altitude: f64, pressure_ratio: f64) -> f64 {
        if apparent_altitude < -0.5 {
            return 0.0;
        }

        let tan_h = (apparent_altitude + 7.31 / (apparent_altitude + 4.4))
            .to_radians()
            .tan();

        let correction_deg = (1.0 / tan_h) / 60.0 * pressure_ratio;
        debug_assert!(
            correction_deg >= 0.0,
            "Atmospheric refraction correction must not be negative (elevation = {}°)",
            apparent_altitude
        );

        correction_deg.clamp(0.0, constants::ATMOSPHERIC_REFRACTION_MAX)
    }
}

/// Apparent solar position (altitude and azimuth) at a given instant.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolarPosition {
    altitude: f64,
    azimuth: f64,
}

impl SolarPosition {
    /// The true altitude for a sine of it.
    ///
    /// `asin` is `NaN` outside `[-1, 1]`, and the sum of two rounded products can land a
    /// few ulps past `1` when the sun is at the zenith — which would make the altitude
    /// `NaN` for that instant and, with it, the frame match.
    fn altitude_from_sin(sin_altitude: f64) -> f64 {
        sin_altitude.clamp(-1.0, 1.0).asin().to_degrees()
    }

    #[inline]
    pub fn new(position: &Position, date_time: &UtcDateTime) -> Self {
        let ctx = SolarContext::new(position, date_time);
        Self {
            altitude: Self::altitude_from_context(&ctx),
            azimuth: Self::azimuth_from_context(&ctx),
        }
    }

    fn altitude_from_context(ctx: &SolarContext) -> f64 {
        let sin_alt = ctx.latitude_rad.sin() * ctx.declination_rad.sin()
            + ctx.latitude_rad.cos() * ctx.declination_rad.cos() * ctx.hour_angle_rad.cos();

        let true_altitude = SolarPosition::altitude_from_sin(sin_alt);

        true_altitude + SolarCalc::atmospheric_refraction(true_altitude, ctx.pressure_ratio)
    }

    fn azimuth_from_context(ctx: &SolarContext) -> f64 {
        let sin_h = ctx.hour_angle_rad.sin();
        let cos_h = ctx.hour_angle_rad.cos();
        let sin_phi = ctx.latitude_rad.sin();
        let cos_phi = ctx.latitude_rad.cos();
        let tan_delta = ctx.declination_rad.tan();

        let denominator = cos_h * sin_phi - tan_delta * cos_phi;

        let azimuth_deg = (sin_h.atan2(denominator).to_degrees() + 360.0) % 360.0;

        (azimuth_deg + 180.0) % 360.0
    }

    pub(crate) fn altitude(&self) -> f64 {
        self.altitude
    }

    pub(crate) fn azimuth(&self) -> f64 {
        self.azimuth
    }
}

/// A solar angle entry from a theme's `solar.json` or `theme.toml`.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SolarAngle {
    index: u8,
    altitude: f64,
    azimuth: f64,
}

impl SolarAngle {
    pub(crate) fn new(index: u8, altitude: f64, azimuth: f64) -> Self {
        Self {
            index,
            altitude,
            azimuth,
        }
    }

    pub fn index(&self) -> u8 {
        self.index
    }

    pub fn altitude(&self) -> f64 {
        self.altitude
    }

    pub fn azimuth(&self) -> f64 {
        self.azimuth
    }
}

#[cfg(test)]
mod tests {
    use time::Month;

    use super::*;

    // ── Position tests ──────────────────────────────────────────────────────

    #[test]
    fn test_position_new() {
        let pos = Position::new(45.0, 90.0, 43.5).unwrap();
        assert_eq!(pos.latitude(), 45.0);
        assert_eq!(pos.longitude(), 90.0);
        assert_eq!(pos.altitude(), 43.5);
    }

    #[test]
    fn pressure_falls_with_observer_elevation() {
        assert_eq!(pressure_ratio(0.0), 1.0);
        assert!(pressure_ratio(3_000.0) < 0.75);
        assert_eq!(pressure_ratio(f64::NAN), 1.0);
        assert_eq!(pressure_ratio(f64::INFINITY), 1.0);
    }

    /// The observer's elevation has to reach the result: a higher station sees less
    /// refraction, so the apparent sun is lower there at the same instant.
    #[test]
    fn a_higher_observer_sees_less_refraction() {
        let date = UtcDateTime::new(2026, Month::March, 20, 6, 10, 0).unwrap();
        let sea_level = SolarPosition::new(&Position::new(0.0, 0.0, 0.0).unwrap(), &date);
        let mountain = SolarPosition::new(&Position::new(0.0, 0.0, 3_000.0).unwrap(), &date);

        assert!(
            mountain.altitude() < sea_level.altitude(),
            "sea level = {}, 3000 m = {}",
            sea_level.altitude(),
            mountain.altitude()
        );
    }

    #[test]
    fn test_position_new_invalid() {
        assert!(Position::new(91.0, 90.0, 43.5).is_err());
        assert!(Position::new(45.0, 181.0, 43.5).is_err());
        assert!(Position::new(-91.0, 0.0, 43.5).is_err());
        assert!(Position::new(0.0, -181.0, 43.5).is_err());
    }

    #[test]
    fn test_position_validation() {
        assert!(Position::is_valid_latitude(90.0));
        assert!(Position::is_valid_latitude(-90.0));
        assert!(Position::is_valid_latitude(0.0));
        assert!(!Position::is_valid_latitude(90.1));
        assert!(!Position::is_valid_latitude(-90.1));

        assert!(Position::is_valid_longitude(180.0));
        assert!(Position::is_valid_longitude(-180.0));
        assert!(Position::is_valid_longitude(0.0));
        assert!(!Position::is_valid_longitude(180.1));
        assert!(!Position::is_valid_longitude(-180.1));
    }

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.latitude(), 0.0);
        assert_eq!(pos.longitude(), 0.0);
    }

    #[test]
    fn test_position_display() {
        let pos = Position::from_raw_position(45.0, 90.0, 43.5);
        assert_eq!(format!("{pos}"), "Position(lat: 45, lng: 90, alt: 43.5)");
    }

    // ── GeographicPositionProvider tests ────────────────────────────────────

    struct MockPositionProvider {
        position: Position,
    }

    impl PositionProvider for MockPositionProvider {
        fn get_current_position(&self) -> DwallResult<Position> {
            Ok(self.position)
        }

        fn check_location_permission(&self) -> DwallResult<()> {
            Ok(())
        }
    }

    /// A backend that counts how often it was really asked.
    struct CountingPositionProvider {
        position: Position,
        calls: std::rc::Rc<std::cell::Cell<u32>>,
    }

    impl PositionProvider for CountingPositionProvider {
        fn get_current_position(&self) -> DwallResult<Position> {
            self.calls.set(self.calls.get() + 1);

            Ok(self.position)
        }

        fn check_location_permission(&self) -> DwallResult<()> {
            Ok(())
        }
    }

    // ── The position cache's lifetime ───────────────────────────────────────

    #[test]
    fn the_position_cache_ttl_follows_the_configured_minutes() {
        assert_eq!(position_cache_ttl(0), Duration::ZERO);
        assert_eq!(position_cache_ttl(1), Duration::from_secs(60));
        assert_eq!(position_cache_ttl(30), Duration::from_secs(1800));
    }

    /// The regression test for the abort: `60 * minutes` used to overflow, and the
    /// resulting duration panicked the addition inside the cache.
    #[test]
    fn the_position_cache_ttl_is_bounded() {
        let bounded = position_cache_ttl(u64::MAX);

        assert!(bounded <= Duration::from_secs(24 * 365 * 3600));
        assert!(
            bounded > Duration::from_secs(60),
            "a value that large is still a cache lifetime, not zero"
        );
    }

    /// A zero-minute cache means "ask every time", which is a usable configuration.
    #[test]
    fn a_zero_minute_cache_reads_the_backend_every_time() {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let source = PositionSource::Automatic {
            update_on_each_calculation: false,
            cache_minutes: 0,
        };
        let provider = GeographicPositionProvider::new(
            &source,
            CountingPositionProvider {
                position: Position::from_raw_position(1.0, 2.0, 3.0),
                calls: std::rc::Rc::clone(&calls),
            },
        );

        provider.get_current_position().unwrap();
        provider.get_current_position().unwrap();

        assert_eq!(calls.get(), 2, "a zero-minute cache must not serve a value");
    }

    #[test]
    fn a_cache_window_serves_the_value_it_already_read() {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let source = PositionSource::Automatic {
            update_on_each_calculation: false,
            cache_minutes: 30,
        };
        let provider = GeographicPositionProvider::new(
            &source,
            CountingPositionProvider {
                position: Position::from_raw_position(1.0, 2.0, 3.0),
                calls: std::rc::Rc::clone(&calls),
            },
        );

        // The cache in `utils` is a thread-local shared by every test on this thread,
        // so the assertion is a delta rather than an absolute count.
        provider.get_current_position().unwrap();
        let after_first = calls.get();

        provider.get_current_position().unwrap();

        assert_eq!(calls.get(), after_first, "the window must not expire yet");
    }

    #[test]
    fn test_provider_manual_coordinates() {
        let coord_source = PositionSource::Manual {
            latitude: 45.0,
            longitude: 90.0,
            altitude: 43.5,
        };
        let mock_provider = MockPositionProvider {
            position: Position::from_raw_position(0.0, 0.0, 0.0),
        };
        let provider = GeographicPositionProvider::new(&coord_source, mock_provider);

        let pos = provider.get_current_position().unwrap();
        assert_eq!(pos.latitude(), 45.0);
        assert_eq!(pos.longitude(), 90.0);
        assert_eq!(pos.altitude(), 43.5);
    }

    #[test]
    fn test_provider_with_mock_position_provider() {
        let coord_source = PositionSource::Automatic {
            update_on_each_calculation: true,
            cache_minutes: 5,
        };
        let expected_position = Position::from_raw_position(40.0, 116.0, 50.0);
        let mock_provider = MockPositionProvider {
            position: expected_position,
        };
        let provider = GeographicPositionProvider::new(&coord_source, mock_provider);

        let pos = provider.get_current_position().unwrap();
        assert_eq!(pos.latitude(), 40.0);
        assert_eq!(pos.longitude(), 116.0);
        assert_eq!(pos.altitude(), 50.0);
    }

    // ── Solar position calculation tests ────────────────────────────────────

    #[test]
    fn solar_position_equinox_equator_noon() {
        let position = Position::from_raw_position(0.0, 0.0, 0.0);
        let dt = UtcDateTime::new(2026, Month::March, 20, 12, 0, 0).unwrap();
        let solar = SolarPosition::new(&position, &dt);

        assert!(
            solar.altitude() > 85.0,
            "altitude should be near 90°, got {}",
            solar.altitude()
        );
        assert!(
            solar.altitude() <= 90.0,
            "altitude should not exceed 90°, got {}",
            solar.altitude()
        );
    }

    #[test]
    fn solar_position_equator_midnight() {
        let position = Position::from_raw_position(0.0, 0.0, 0.0);
        let dt = UtcDateTime::new(2026, Month::March, 20, 0, 0, 0).unwrap();
        let solar = SolarPosition::new(&position, &dt);

        assert!(
            solar.altitude() < 0.0,
            "altitude should be below horizon at midnight, got {}",
            solar.altitude()
        );
    }

    #[test]
    fn solar_position_arctic_summer_solstice() {
        let position = Position::from_raw_position(66.5, 0.0, 0.0);
        let dt = UtcDateTime::new(2026, Month::June, 21, 0, 0, 0).unwrap();
        let solar = SolarPosition::new(&position, &dt);

        assert!(
            solar.altitude() > 0.0,
            "altitude should be above horizon (midnight sun), got {}",
            solar.altitude()
        );
    }

    #[test]
    fn solar_position_high_latitude_winter() {
        let position = Position::from_raw_position(60.0, 0.0, 0.0);
        let dt = UtcDateTime::new(2026, Month::December, 21, 12, 0, 0).unwrap();
        let solar = SolarPosition::new(&position, &dt);

        assert!(
            solar.altitude() < 10.0,
            "altitude should be low in winter, got {}",
            solar.altitude()
        );
    }

    #[test]
    fn azimuth_always_in_valid_range() {
        let position = Position::from_raw_position(45.0, 0.0, 0.0);
        for hour in 0..24u8 {
            let dt = UtcDateTime::new(2026, Month::June, 21, hour, 0, 0).unwrap();
            let solar = SolarPosition::new(&position, &dt);

            assert!(
                solar.azimuth() >= 0.0 && solar.azimuth() < 360.0,
                "azimuth {} should be in [0, 360) at hour {}",
                solar.azimuth(),
                hour
            );
        }
    }

    #[test]
    fn altitude_always_in_valid_range() {
        let position = Position::from_raw_position(45.0, 0.0, 0.0);
        for hour in 0..24u8 {
            let dt = UtcDateTime::new(2026, Month::June, 21, hour, 0, 0).unwrap();
            let solar = SolarPosition::new(&position, &dt);

            assert!(
                solar.altitude() >= -90.0 && solar.altitude() <= 90.0,
                "altitude {} should be in [-90, 90] at hour {}",
                solar.altitude(),
                hour
            );
        }
    }

    #[test]
    fn julian_day_known_value() {
        let dt = UtcDateTime::new(2000, Month::January, 1, 12, 0, 0).unwrap();
        let jd = SolarCalc::julian_day(&dt);
        assert!(
            (jd - 2451545.0).abs() < 0.001,
            "Julian Day should be 2451545.0, got {}",
            jd
        );
    }

    #[test]
    fn julian_century_at_epoch() {
        let dt = UtcDateTime::new(2000, Month::January, 1, 12, 0, 0).unwrap();
        let t = SolarCalc::julian_century_t(&dt);
        assert!(
            t.abs() < 0.001,
            "Julian century should be 0 at epoch, got {}",
            t
        );
    }

    #[test]
    fn snapshot_solar_position_24h() {
        let position = Position::from_raw_position(45.0, 116.0, 0.0); // Beijing
        let mut results: Vec<(u8, f64, f64)> = Vec::new();

        for hour in 0..24u8 {
            let dt = UtcDateTime::new(2026, Month::June, 21, hour, 0, 0).unwrap();
            let solar = SolarPosition::new(&position, &dt);
            results.push((hour, solar.altitude(), solar.azimuth()));
        }

        insta::assert_json_snapshot!("solar_position_24h_beijing", results);
    }

    // ── Physics invariants ──────────────────────────────────────────────────
    //
    // These check the geometry against facts that hold for any correct solar
    // implementation — the meridian crossing, the solstice triangle, the obliquity and
    // the equation of time — so they catch a wrong sign or a wrong formula that a
    // self-consistent set of snapshots cannot. They are *not* a comparison against a
    // reference ephemeris: that needs a citable table of published values, which this
    // checkout does not carry.

    /// Every day of 2026, so an annual curve can be walked without an ordinal-date API.
    fn days_of_year() -> Vec<(Month, u8)> {
        // 2026 is not a leap year.
        const LENGTHS: [(Month, u8); 12] = [
            (Month::January, 31),
            (Month::February, 28),
            (Month::March, 31),
            (Month::April, 30),
            (Month::May, 31),
            (Month::June, 30),
            (Month::July, 31),
            (Month::August, 31),
            (Month::September, 30),
            (Month::October, 31),
            (Month::November, 30),
            (Month::December, 31),
        ];

        LENGTHS
            .into_iter()
            .flat_map(|(month, days)| (1..=days).map(move |day| (month, day)))
            .collect()
    }

    const BEIJING: (f64, f64, f64) = (39.9042, 116.4074, 43.5);

    fn position_at((latitude, longitude, altitude): (f64, f64, f64)) -> Position {
        Position::from_raw_position(latitude, longitude, altitude)
    }

    /// The instant of the day's highest sun, as seconds of the UTC day.
    ///
    /// Scanned in five-minute steps and then refined in five-second steps around that
    /// peak, because a coarse sample can sit minutes away from the meridian and the
    /// azimuth moves quickly at a high sun.
    fn meridian_crossing(position: &Position, year: u16, month: Month, day: u8) -> (f64, f64, u32) {
        let altitude_at = |second: u32| {
            let utc = UtcDateTime::new(
                year,
                month,
                day,
                (second / 3600) as u8,
                ((second % 3600) / 60) as u8,
                (second % 60) as u8,
            )
            .unwrap();

            SolarPosition::new(position, &utc)
        };

        let coarse = (0..86_400).step_by(300).max_by(|a, b| {
            altitude_at(*a)
                .altitude()
                .total_cmp(&altitude_at(*b).altitude())
        });
        let coarse = coarse.expect("a day has samples");

        let finest = (coarse.saturating_sub(300)..=(coarse + 300).min(86_399))
            .step_by(5)
            .max_by(|a, b| {
                altitude_at(*a)
                    .altitude()
                    .total_cmp(&altitude_at(*b).altitude())
            })
            .expect("the window has samples");

        let solar = altitude_at(finest);

        (solar.altitude(), solar.azimuth(), finest)
    }

    /// `asin` of a value that rounding pushed just past ±1 is `NaN`, and the path that
    /// computes the sine cannot be pushed there from outside, so the helper is tested
    /// directly.
    #[test]
    fn altitude_is_clamped_when_the_sine_rounds_past_one() {
        assert_eq!(
            SolarPosition::altitude_from_sin(1.0 + f64::EPSILON),
            90.0,
            "a sine a rounding step above one is the zenith, not NaN"
        );
        assert_eq!(SolarPosition::altitude_from_sin(-1.0 - f64::EPSILON), -90.0);
        assert_eq!(SolarPosition::altitude_from_sin(0.0), 0.0);

        // The clamp is a range guard, not a NaN filter: a `NaN` stays one, and the
        // colour-scheme layer is what decides not to act on it.
        assert!(SolarPosition::altitude_from_sin(f64::NAN).is_nan());
    }

    /// The daily maximum is the meridian crossing, and the sun is due south there in the
    /// northern hemisphere.
    #[test]
    fn the_highest_sun_of_the_day_is_due_south() {
        let (altitude, azimuth, _) =
            meridian_crossing(&position_at(BEIJING), 2026, Month::June, 21);

        assert!(
            (azimuth - 180.0).abs() < 0.5,
            "the noon sun above Beijing is due south, got {azimuth}°"
        );
        assert!(
            altitude > 70.0,
            "the June sun above Beijing reaches 73°, got {altitude}°"
        );
    }

    /// The same crossing is due *north* below the equator, which no amount of internal
    /// consistency would get right if the azimuth formula's hemisphere handling broke.
    #[test]
    fn the_highest_sun_of_the_day_is_due_north_below_the_equator() {
        // Sydney, in its winter: the sun stays low and crosses to the north.
        let (altitude, azimuth, _) = meridian_crossing(
            &position_at((-33.8688, 151.2093, 58.0)),
            2026,
            Month::June,
            21,
        );

        assert!(
            azimuth < 0.5 || azimuth > 359.5,
            "the winter noon sun above Sydney is due north, got {azimuth}°"
        );
        assert!(
            altitude > 25.0,
            "it still climbs to about 33°, got {altitude}°"
        );
    }

    /// Solar noon sits where geography puts it: 12:00 minus the longitude offset from the
    /// time zone's meridian, minus the equation of time.
    ///
    /// November is the case worth pinning, because the equation of time is then at its
    /// positive extreme (+16.4 min), so the crossing lands near 11:58 in Beijing. An
    /// implementation that got that sign backwards would still agree with itself
    /// everywhere, but would put its noon half an hour later — which is what this catches.
    #[test]
    fn the_sun_crosses_the_meridian_at_the_clock_time_the_longitude_implies() {
        // 116.4074°E is 3.6° west of the 120°E meridian of UTC+8, so longitude alone puts
        // solar noon at 12:14:24; the equation of time moves it to about 11:58.
        let expected_utc_seconds = (11 * 3600 + 58 * 60) as f64 - 8.0 * 3600.0;
        let expected_utc_seconds = if expected_utc_seconds < 0.0 {
            expected_utc_seconds + 86_400.0
        } else {
            expected_utc_seconds
        };

        let (_, _, crossing) = meridian_crossing(&position_at(BEIJING), 2026, Month::November, 3);
        let drift = (f64::from(crossing) - expected_utc_seconds).abs();

        assert!(
            drift < 8.0 * 60.0,
            "solar noon should be near 11:58 Beijing time ({} UTC seconds), got {crossing}",
            expected_utc_seconds
        );
    }

    /// At the solstices the noon sun stands `90° − |latitude − declination|` above the
    /// horizon, with the declination at the obliquity of the ecliptic (23.44°). The
    /// declination comes from the implementation, so this pins it against the one number
    /// astronomy fixes for that date.
    #[test]
    fn the_noon_altitude_matches_the_solstice_triangle() {
        const OBLIQUITY: f64 = 23.44;

        for (latitude, month, day) in [
            (0.0, Month::June, 21),
            (BEIJING.0, Month::June, 21),
            (55.0, Month::June, 21),
            (BEIJING.0, Month::December, 21),
        ] {
            let expected = 90.0
                - (latitude
                    - if month == Month::June {
                        OBLIQUITY
                    } else {
                        -OBLIQUITY
                    })
                .abs();
            let (altitude, _, _) =
                meridian_crossing(&position_at((latitude, BEIJING.1, 0.0)), 2026, month, day);

            assert!(
                (altitude - expected).abs() < 0.5,
                "at {latitude}° on {day} {month:?} the noon sun is {expected}° up, got {altitude}°"
            );
        }
    }

    #[test]
    fn the_declination_stays_within_the_obliquity_and_reaches_it() {
        let mut highest: f64 = f64::MIN;
        let mut lowest: f64 = f64::MAX;

        for (month, day) in days_of_year() {
            let utc = UtcDateTime::new(2026, month, day, 0, 0, 0).unwrap();
            let t = SolarCalc::julian_century_t(&utc);
            let declination = SolarCalc::solar_declination(t, SolarCalc::earth_axial_tilt(t));

            highest = highest.max(declination);
            lowest = lowest.min(declination);

            assert!(
                (-23.45..=23.45).contains(&declination),
                "the sun never stands more than the obliquity off the equator, got {declination}°"
            );
        }

        assert!(
            (23.4..=23.45).contains(&highest),
            "the June solstice has to reach the obliquity, got {highest}°"
        );
        assert!(
            (-23.45..=-23.4).contains(&lowest),
            "the December solstice has to reach it the other way, got {lowest}°"
        );
    }

    /// The equation of time swings between about −14 minutes in February and +16 in
    /// November. The range catches a scale error, and the months catch a sign one.
    #[test]
    fn the_equation_of_time_stays_within_a_quarter_of_an_hour() {
        let mut extremes: Vec<(f64, Month)> = Vec::new();

        for (month, day) in days_of_year() {
            let utc = UtcDateTime::new(2026, month, day, 12, 0, 0).unwrap();
            let t = SolarCalc::julian_century_t(&utc);
            let minutes =
                SolarCalc::solar_time_correction(t, SolarCalc::earth_axial_tilt(t)) * 60.0;

            assert!(
                (-17.0..=17.0).contains(&minutes),
                "the equation of time stays inside ±17 minutes, got {minutes}"
            );

            extremes.push((minutes, month));
        }

        let (lowest, lowest_month) = extremes
            .iter()
            .copied()
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .unwrap();
        let (highest, highest_month) = extremes
            .iter()
            .copied()
            .max_by(|a, b| a.0.total_cmp(&b.0))
            .unwrap();

        assert!(
            (-15.0..=-13.5).contains(&lowest),
            "the sundial runs slowest in early February, got {lowest} min"
        );
        assert_eq!(lowest_month, Month::February);
        assert!(
            (15.5..=17.0).contains(&highest),
            "and fastest in early November, got {highest} min"
        );
        assert_eq!(highest_month, Month::November);
    }

    // ── Boundary inputs ─────────────────────────────────────────────────────

    #[test]
    fn position_new_rejects_nan_and_infinity() {
        assert!(matches!(
            Position::new(f64::NAN, 0.0, 0.0),
            Err(crate::error::DwallError::Position(
                CoordinateError::InvalidLatitude(latitude)
            )) if latitude.is_nan()
        ));
        assert!(matches!(
            Position::new(0.0, f64::INFINITY, 0.0),
            Err(crate::error::DwallError::Position(
                CoordinateError::InvalidLongitude(longitude)
            )) if longitude.is_infinite()
        ));
    }

    #[test]
    fn solar_position_at_the_south_pole_and_longitude_dateline() {
        // The June solstice, so the northern pole is at its highest sun and the
        // southern one at its lowest.
        let date = UtcDateTime::new(2026, Month::June, 21, 12, 0, 0).unwrap();

        for longitude in [-180.0, 180.0] {
            let south =
                SolarPosition::new(&Position::from_raw_position(-90.0, longitude, 0.0), &date);
            let north =
                SolarPosition::new(&Position::from_raw_position(90.0, longitude, 0.0), &date);

            for solar in [&south, &north] {
                assert!(
                    (-90.0..=90.0).contains(&solar.altitude()),
                    "altitude {} is outside [-90, 90]",
                    solar.altitude()
                );
                assert!(
                    (0.0..360.0).contains(&solar.azimuth()),
                    "azimuth {} is outside [0, 360)",
                    solar.azimuth()
                );
            }

            // The obliquity at the solstice, and the two poles as mirror images of
            // each other apart from atmospheric refraction at the lit one.
            assert!(
                (north.altitude() - 23.44).abs() < 0.1,
                "the June sun stands at the obliquity above the north pole, got {}",
                north.altitude()
            );
            assert!(
                (south.altitude() + 23.44).abs() < 0.1,
                "and the same below the south pole, got {}",
                south.altitude()
            );
            assert!(
                (north.altitude() + south.altitude()).abs() < 0.1,
                "the poles are opposites: north {} south {}",
                north.altitude(),
                south.altitude()
            );
        }
    }

    #[test]
    fn equator_sun_rises_and_sets_once_per_day() {
        let position = Position::from_raw_position(0.0, 0.0, 0.0);
        let mut rises = 0u32;
        let mut sets = 0u32;
        let mut was_above: Option<bool> = None;

        for second in (0..86_400u32).step_by(60) {
            let date = UtcDateTime::new(
                2026,
                Month::June,
                21,
                (second / 3600) as u8,
                ((second % 3600) / 60) as u8,
                0,
            )
            .unwrap();
            let above = SolarPosition::new(&position, &date).altitude() > 0.0;

            if let Some(previous) = was_above {
                if above && !previous {
                    rises += 1;
                }
                if !above && previous {
                    sets += 1;
                }
            }

            was_above = Some(above);
        }

        assert_eq!(rises, 1, "the equator sees exactly one sunrise a day");
        assert_eq!(sets, 1, "and exactly one sunset");
    }

    use proptest::prelude::*;

    proptest! {
        /// The ranges are a contract the frame matcher relies on: an altitude outside
        /// [-90, 90] or an azimuth outside [0, 360) means the geometry broke, and a
        /// NaN silently selects nothing. The fixed positions above only sample a few
        /// coordinates, so the boundary cases are left to the generator.
        #[test]
        fn solar_position_stays_in_range_for_any_position_and_instant(
            latitude in -90.0f64..=90.0,
            longitude in -180.0f64..=180.0,
            seconds in 0u64..=4_102_444_800,
        ) {
            let position = Position::from_raw_position(latitude, longitude, 0.0);
            let date_time = UtcDateTime::from_timestamp(seconds);
            let solar = SolarPosition::new(&position, &date_time);

            prop_assert!(
                solar.altitude().is_finite(),
                "altitude {} at ({latitude}, {longitude}) @ {seconds}",
                solar.altitude()
            );
            prop_assert!(
                (-90.0..=90.0).contains(&solar.altitude()),
                "altitude {} at ({latitude}, {longitude}) @ {seconds}",
                solar.altitude()
            );
            prop_assert!(
                solar.azimuth().is_finite(),
                "azimuth {} at ({latitude}, {longitude}) @ {seconds}",
                solar.azimuth()
            );
            prop_assert!(
                (0.0..360.0).contains(&solar.azimuth()),
                "azimuth {} at ({latitude}, {longitude}) @ {seconds}",
                solar.azimuth()
            );
        }
    }
}
