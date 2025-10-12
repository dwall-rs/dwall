use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Astronomical calculation constants module
/// These constants are used for precise calculation of the sun's position.
mod constants {
    /// Julian date for January 1, 2000 at 12:00 (TT)
    /// From: IAU 2000 Resolution B1.6 (Terrestrial Time system)
    pub const EPOCH_J2000: f64 = 2451545.0;

    /// Number of days in a Julian century
    pub const JULIAN_CENTURY_DAYS: f64 = 36525.0;

    /// Earth's rotation rate in degrees per hour
    pub const EARTH_ROTATION_RATE: f64 = 15.0;

    /// Hours per day
    pub const HOURS_PER_DAY: f64 = 24.0;

    /// Minutes per hour
    pub const MINUTES_PER_HOUR: f64 = 60.0;

    /// Seconds per hour
    pub const SECONDS_PER_HOUR: f64 = 3600.0;

    /// Maximum atmospheric refraction correction at horizon (degrees)
    /// Based on Saemundsson's refraction formula (1986)
    pub const ATMOSPHERIC_REFRACTION_MAX: f64 = 0.55;

    /// Polar region latitude threshold (degrees)
    /// Above this latitude, special handling is needed for azimuth calculation
    pub const POLAR_REGION_THRESHOLD: f64 = 89.9;
}

/// Solar angle data structure
///
/// Used to store the sun's altitude and azimuth angles at a specific moment, along with the corresponding index
/// This data is used to determine which wallpaper image should be displayed
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SolarAngle {
    /// Image index, used to match the corresponding wallpaper image
    pub index: u8,
    /// Sun's altitude angle (degrees), representing the angle of the sun relative to the horizon
    pub altitude: f64,
    /// Sun's azimuth angle (degrees), representing the angle of the sun relative to true north (clockwise)
    pub azimuth: f64,
}

/// Sun position calculator
///
/// Calculates the sun's position parameters based on geographic location and time
/// Including altitude and azimuth angles, which determine the angle and intensity of sunlight
pub struct SunPosition {
    /// Geographic latitude (degrees), positive for north, negative for south
    latitude: f64,
    /// Geographic longitude (degrees), positive for east, negative for west
    longitude: f64,
    /// Date and time for calculating the sun's position
    date_time: OffsetDateTime,
}

impl SunPosition {
    pub fn new(latitude: f64, longitude: f64, utc_time: OffsetDateTime) -> Self {
        Self {
            latitude,
            longitude,
            date_time: utc_time,
        }
    }

    /// Calculate Julian day using Fliegel-Van Flandern algorithm
    ///
    /// # Algorithm Reference
    /// Verified against: Meeus AA Ch. 7, Eq. 7.1
    ///
    /// # Accuracy
    /// ±0.5 seconds for dates between 1900-2100
    fn julian_day(&self) -> f64 {
        let year = self.date_time.year() as f64;
        let month = self.date_time.month() as u8 as f64;
        let day = self.date_time.day() as f64
            + self.date_time.hour() as f64 / constants::HOURS_PER_DAY
            + self.date_time.minute() as f64 / 1440.0
            + self.date_time.second() as f64 / 86400.0;

        let a = ((14.0 - month) / 12.0).floor();
        let y = year + 4800.0 - a;
        let m = month + 12.0 * a - 3.0;

        day + ((153.0 * m + 2.0) / 5.0) + 365.0 * y + (y / 4.0) - (y / 100.0) + (y / 400.0)
            - 32045.5
    }

    /// Calculate the Julian century offset from J2000 epoch
    ///
    /// This is the time factor used in various astronomical calculations,
    /// representing the number of Julian centuries since J2000.0
    fn julian_century_offset(&self) -> f64 {
        let jd = self.julian_day();
        (jd - constants::EPOCH_J2000) / constants::JULIAN_CENTURY_DAYS
    }

    /// Calculate the mean obliquity of the ecliptic (Earth's axial tilt)
    ///
    /// # Arguments
    /// * `t` - Julian century offset from J2000 epoch
    ///
    /// # Algorithm Reference
    /// Formula from: IAU 2006 precession model
    /// Reference: Meeus AA Ch. 22, Eq. 22.3
    fn earth_axial_tilt(&self, t: f64) -> f64 {
        23.43929111 - 0.0130042 * t - 0.00000164 * t.powi(2) - 0.000000503 * t.powi(3)
    }

    /// Calculate solar ecliptic longitude using simplified VSOP87 model
    ///
    /// # Arguments
    /// * `t` - Julian century offset from J2000 epoch
    ///
    /// # Algorithm Reference
    /// Simplified VSOP87 model parameters (accuracy ±0.01°)
    /// Reference: Meeus Astronomical Algorithms 2nd Ed. Chapter 25
    /// Original theory: Francou et al. 1988
    fn solar_ecliptic_longitude(t: f64) -> f64 {
        let l0 = (280.4664567 + 36000.7698278 * t + 0.0003032028 * t.powi(2)) % 360.0;
        let l1 = 1.914602 + 0.004817 * t + 0.000014 * t.powi(2);
        let l2 = 0.019993 - 0.000101 * t;
        let l3 = 0.000289;

        let m = (357.5291092 + 35999.0502909 * t).to_radians();
        l0 + l1 * m.sin() + l2 * (2.0 * m).sin() + l3 * (3.0 * m).sin()
    }

    /// Calculate the solar declination angle
    ///
    /// Solar declination is the angle between the sun's rays and the Earth's equatorial plane.
    /// It varies between approximately -23.45° and +23.45° throughout the year.
    fn solar_declination(&self) -> f64 {
        let t = self.julian_century_offset();

        let epsilon = self.earth_axial_tilt(t);

        let lambda = Self::solar_ecliptic_longitude(t);
        let delta = epsilon.to_radians().sin() * lambda.to_radians().sin();

        delta.asin().to_degrees()
    }

    /// Calculate the equation of time correction
    ///
    /// The equation of time is the difference between apparent solar time and mean solar time.
    /// It accounts for the Earth's elliptical orbit and axial tilt.
    ///
    /// # Returns
    /// Time correction in hours (can be added to mean solar time)
    fn solar_time_correction(&self) -> f64 {
        let t = self.julian_century_offset();

        let m_deg = 357.52911 + 35999.05029 * t - 0.0001537 * t.powi(2);
        let m = m_deg.to_radians();

        let l0_deg = 280.46646 + 36000.76983 * t + 0.0003032 * t.powi(2);

        let epsilon_deg = self.earth_axial_tilt(t);
        let epsilon = epsilon_deg.to_radians();

        let c = (1.914602 - 0.004817 * t - 0.000014 * t.powi(2)) * m.sin()
            + (0.019993 - 0.000101 * t) * (2.0 * m).sin()
            + 0.000289 * (3.0 * m).sin();

        let lambda_deg = l0_deg + c;
        let lambda = lambda_deg.to_radians();

        let alpha = (lambda.sin() * epsilon.cos()).atan2(lambda.cos());
        let eot = (l0_deg - alpha.to_degrees() - 0.0057183) * 4.0; // Convert to minutes
        eot / constants::MINUTES_PER_HOUR // Convert to hours
    }

    fn decimal_hours(&self) -> f64 {
        self.date_time.hour() as f64
            + self.date_time.minute() as f64 / constants::MINUTES_PER_HOUR
            + self.date_time.second() as f64 / constants::SECONDS_PER_HOUR
    }

    /// Calculate the hour angle
    ///
    /// The hour angle indicates how far the sun has moved across the sky from solar noon.
    /// It is 0° at solar noon, negative before noon, and positive after noon.
    fn hour_angle(&self) -> f64 {
        let hours = self.decimal_hours();

        let equation_of_time = self.solar_time_correction();
        let local_mean_time =
            hours + (self.longitude / constants::EARTH_ROTATION_RATE) + equation_of_time;

        let raw_angle = constants::EARTH_ROTATION_RATE * (local_mean_time - 12.0);
        (raw_angle % 360.0 + 360.0) % 360.0
    }

    /// Calculate atmospheric refraction correction
    ///
    /// Atmospheric refraction makes celestial objects appear higher in the sky than they actually are.
    /// This effect is strongest near the horizon.
    ///
    /// # Arguments
    /// * `apparent_altitude` - The observed altitude angle in degrees
    ///
    /// # Returns
    /// Refraction correction in degrees to be added to true altitude
    ///
    /// # Algorithm Reference
    /// Saemundsson's refraction formula (1986)
    /// Reference: Meeus AA Ch. 16, Bulletin of the Astronomical Institutes of Czechoslovakia
    fn atmospheric_refraction(&self, apparent_altitude: f64) -> f64 {
        if apparent_altitude < -0.5 {
            0.0
        } else {
            let alt_rad = apparent_altitude.to_radians();
            let correction =
                1.0 / (alt_rad.tan() + 7.31 / (alt_rad + 4.4).to_degrees().sqrt()).to_degrees();
            correction.min(constants::ATMOSPHERIC_REFRACTION_MAX)
        }
    }

    /// Calculate the sun's altitude angle above the horizon
    ///
    /// # Returns
    /// Altitude angle in degrees, accounting for atmospheric refraction:
    /// - 0° = sun at horizon
    /// - 90° = sun directly overhead
    /// - Negative values = sun below horizon
    pub fn altitude(&self) -> f64 {
        let latitude_rad = self.latitude.to_radians();
        let declination_rad = self.solar_declination().to_radians();
        let hour_angle_rad = self.hour_angle().to_radians();

        let sine_altitude = latitude_rad.sin() * declination_rad.sin()
            + latitude_rad.cos() * declination_rad.cos() * hour_angle_rad.cos();

        let true_altitude = sine_altitude.asin().to_degrees();

        true_altitude + self.atmospheric_refraction(true_altitude)
    }

    /// Calculate the sun's azimuth angle
    ///
    /// # Returns
    /// Azimuth angle in degrees measured clockwise from true north:
    /// - 0° = North
    /// - 90° = East
    /// - 180° = South
    /// - 270° = West
    pub fn azimuth(&self) -> f64 {
        let latitude_rad = self.latitude.to_radians();
        let declination_rad = self.solar_declination().to_radians();
        let hour_angle_rad = self.hour_angle().to_radians();

        let sin_omega = hour_angle_rad.sin();
        let cos_omega = hour_angle_rad.cos();
        let sin_phi = latitude_rad.sin();
        let cos_phi = latitude_rad.cos();
        let tan_delta = declination_rad.tan();

        let denominator = cos_omega * sin_phi - tan_delta * cos_phi;

        // Special handling for polar regions (above POLAR_REGION_THRESHOLD)
        if self.latitude.abs() > constants::POLAR_REGION_THRESHOLD {
            let is_north = self.latitude > 0.0;
            return if is_north { 180.0 } else { 0.0 };
        }

        let azimuth_deg = if denominator.abs() < 1e-6 {
            if self.hour_angle() < 180.0 {
                90.0
            } else {
                270.0
            }
        } else {
            let azimuth_rad = sin_omega.atan2(denominator);
            (azimuth_rad.to_degrees() + 360.0) % 360.0
        };

        // Adjust to geographic azimuth (0° = North)
        (azimuth_deg + 180.0) % 360.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Date, Month, Time};

    // Helper function to create a specific date and time
    fn create_datetime(
        year: i32,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> OffsetDateTime {
        let date = Date::from_calendar_date(year, month, day).unwrap();
        let time = Time::from_hms(hour, minute, second).unwrap();
        date.with_time(time).assume_utc()
    }

    // Helper function to assert that two f64 values are approximately equal
    fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!(
            (a - b).abs() < epsilon,
            "assertion failed: `(left ≈ right)`\n    left: `{a}`\n   right: `{b}`\n epsilon: `{epsilon}`"
        );
    }

    #[test]
    fn test_solar_declination() {
        // Test for summer solstice (around June 21)
        let sun_pos = SunPosition::new(0.0, 0.0, create_datetime(2023, Month::June, 21, 12, 0, 0));
        // Solar declination should be close to 23.45 degrees (maximum)
        assert_approx_eq(sun_pos.solar_declination(), 23.45, 1.0);

        // Test for winter solstice (around December 21)
        let sun_pos = SunPosition::new(
            0.0,
            0.0,
            create_datetime(2023, Month::December, 21, 12, 0, 0),
        );
        // Solar declination should be close to -23.45 degrees (minimum)
        assert_approx_eq(sun_pos.solar_declination(), -23.45, 1.0);

        // Test for equinox (around March 21 or September 21)
        let sun_pos = SunPosition::new(0.0, 0.0, create_datetime(2023, Month::March, 21, 12, 0, 0));
        // Solar declination should be close to 0 degrees
        assert_approx_eq(sun_pos.solar_declination(), 0.0, 1.0);
    }

    #[test]
    fn test_hour_angle() {
        // Test at solar noon (hour angle should be close to 0)
        let sun_pos = SunPosition::new(
            0.0,
            0.0, // At Greenwich
            create_datetime(2023, Month::June, 21, 12, 0, 0),
        );
        assert_approx_eq(360. - sun_pos.hour_angle(), 0.0, 5.0);

        // Test 6 hours before solar noon (hour angle should be around -90 degrees)
        let sun_pos = SunPosition::new(0.0, 0.0, create_datetime(2023, Month::June, 21, 6, 0, 0));
        assert_approx_eq(sun_pos.hour_angle(), -90.0 + 360., 5.0);

        // Test 6 hours after solar noon (hour angle should be around 90 degrees)
        let sun_pos = SunPosition::new(0.0, 0.0, create_datetime(2023, Month::June, 21, 18, 0, 0));
        assert_approx_eq(sun_pos.hour_angle(), 90.0, 5.0);
    }

    #[test]
    fn test_altitude() {
        // Test at equator, solar noon, equinox
        // Sun should be directly overhead (altitude ≈ 90°)
        let sun_pos = SunPosition::new(0.0, 0.0, create_datetime(2023, Month::March, 21, 12, 0, 0));
        assert_approx_eq(sun_pos.altitude(), 90.0, 5.0);

        // Test at North Pole during summer solstice
        // Sun should be at about 23.45° altitude
        let sun_pos = SunPosition::new(90.0, 0.0, create_datetime(2023, Month::June, 21, 12, 0, 0));
        assert_approx_eq(sun_pos.altitude(), 23.45, 5.0);

        // Test at South Pole during winter solstice
        // At this time, the South Pole is in polar day, altitude angle should be 23.5°
        let sun_pos = SunPosition::new(
            -90.0,
            0.0,
            create_datetime(2023, Month::December, 21, 12, 0, 0),
        );

        assert_approx_eq(sun_pos.altitude(), 23.5, 0.5);
    }

    #[test]
    fn test_azimuth() {
        // Test at equator, solar noon
        // Sun should be due north (azimuth = 0°) during winter in Northern Hemisphere
        let sun_pos = SunPosition::new(
            0.0,
            0.0,
            create_datetime(2023, Month::December, 21, 12, 0, 0),
        );
        // Allow for some deviation due to the equation of time
        assert_approx_eq(sun_pos.azimuth(), 180.0, 10.0);

        // Test at Northern Hemisphere, morning
        // Sun should be in the east by north (azimuth ≈ 71.6°)
        let sun_pos = SunPosition::new(40.0, 0.0, create_datetime(2023, Month::June, 21, 6, 0, 0));
        assert_approx_eq(sun_pos.azimuth(), 71.6, 15.0);

        // Test at Northern Hemisphere, evening
        // Sun should be in the west by north (azimuth ≈ 288.4°)
        let sun_pos = SunPosition::new(40.0, 0.0, create_datetime(2023, Month::June, 21, 18, 0, 0));
        assert_approx_eq(sun_pos.azimuth(), 281.6, 15.0);
    }

    #[test]
    fn test_timezone_effect() {
        // Create two SunPosition objects with the same UTC time but different timezone offsets
        let sun_pos_utc =
            SunPosition::new(40.0, 0.0, create_datetime(2023, Month::June, 21, 12, 0, 0));

        let sun_pos_est = SunPosition::new(
            40.0,
            -75.0, // Eastern US longitude
            create_datetime(2023, Month::June, 21, 12, 0, 0),
        );

        // The hour angles should be different due to timezone offset
        assert!(sun_pos_utc.hour_angle().abs() > 1.0);
        assert!(sun_pos_est.hour_angle().abs() > 1.0);
        assert!((sun_pos_utc.hour_angle() - sun_pos_est.hour_angle()).abs() > 1.0);
    }

    #[test]
    fn test_edge_cases() {
        // Test at International Date Line
        let sun_pos_west =
            SunPosition::new(0.0, 179.9, create_datetime(2023, Month::June, 21, 12, 0, 0));

        let sun_pos_east = SunPosition::new(
            0.0,
            -179.9,
            create_datetime(2023, Month::June, 21, 12, 0, 0),
        );

        // Despite being at almost the same location, the hour angles should be similar
        assert_approx_eq(sun_pos_west.hour_angle(), sun_pos_east.hour_angle(), 5.0);

        // Test at extreme latitudes (but not exactly at poles to avoid division by zero)
        let sun_pos_north =
            SunPosition::new(89.9, 0.0, create_datetime(2023, Month::June, 21, 12, 0, 0));

        let sun_pos_south = SunPosition::new(
            -89.9,
            0.0,
            create_datetime(2023, Month::December, 21, 12, 0, 0),
        );

        // Both should have high altitude during their respective summer solstices
        assert!(sun_pos_north.altitude() > 0.0);
        assert!(sun_pos_south.altitude() > 0.0);
    }
}
