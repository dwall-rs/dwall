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
#[derive(Debug, thiserror::Error)]
pub enum GeolocationAccessError {
    #[error("Geolocation permission was denied by the user")]
    Denied,
    #[error("Geolocation permission status is unspecified")]
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
#[derive(Debug, Clone, Copy)]
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
                    self.get_cached_position(Duration::from_secs(60 * position_cache_minutes))
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

struct SolarContext {
    latitude_rad: f64,
    declination_rad: f64,
    hour_angle_rad: f64,
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

    fn atmospheric_refraction(apparent_altitude: f64) -> f64 {
        if apparent_altitude < -0.5 {
            return 0.0;
        }

        let tan_h = (apparent_altitude + 7.31 / (apparent_altitude + 4.4))
            .to_radians()
            .tan();

        let correction_deg = (1.0 / tan_h) / 60.0;
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

        let true_altitude = sin_alt.asin().to_degrees();
        true_altitude + SolarCalc::atmospheric_refraction(true_altitude)
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

/// A solar angle entry from a theme's `solar.json`.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SolarAngle {
    index: u8,
    altitude: f64,
    azimuth: f64,
}

impl SolarAngle {
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
}
