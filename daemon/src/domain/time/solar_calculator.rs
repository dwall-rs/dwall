//! Solar position calculation utilities for astronomical computations

use serde::{Deserialize, Serialize};

use crate::utils::datetime::UtcDateTime;

/// Astronomical calculation constants
mod constants {
    /// Julian date for January 1, 2000 at 12:00 (TT)
    pub(crate) const EPOCH_J2000: f64 = 2451545.0;

    /// Number of days in a Julian century
    pub(crate) const JULIAN_CENTURY_DAYS: f64 = 36525.0;

    /// Earth's rotation rate in degrees per hour
    pub(crate) const EARTH_ROTATION_RATE: f64 = 15.0;

    /// Hours per day
    pub(crate) const HOURS_PER_DAY: f64 = 24.0;

    /// Minutes per hour
    pub(crate) const MINUTES_PER_HOUR: f64 = 60.0;

    /// Seconds per hour
    pub(crate) const SECONDS_PER_HOUR: f64 = 3600.0;

    /// Maximum atmospheric refraction correction at horizon (degrees)
    pub(crate) const ATMOSPHERIC_REFRACTION_MAX: f64 = 0.55;

    /// Polar region latitude threshold (degrees)
    pub(crate) const POLAR_REGION_THRESHOLD: f64 = 89.9;
}

/// Solar angle data structure for wallpaper selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct SolarAngle {
    /// Image index for wallpaper selection
    pub(crate) index: u8,
    /// Sun's altitude angle (degrees)
    pub(crate) altitude: f64,
    /// Sun's azimuth angle (degrees)
    pub(crate) azimuth: f64,
}

/// Solar position calculator for astronomical computations
pub(crate) struct SunPosition {
    latitude: f64,
    longitude: f64,
    date_time: UtcDateTime,
}

impl SunPosition {
    pub(crate) fn new(latitude: f64, longitude: f64, date_time: UtcDateTime) -> Self {
        Self {
            latitude,
            longitude,
            date_time,
        }
    }

    /// Calculate Julian day using Fliegel-Van Flandern algorithm
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
    fn julian_century_offset(&self) -> f64 {
        let jd = self.julian_day();
        (jd - constants::EPOCH_J2000) / constants::JULIAN_CENTURY_DAYS
    }

    /// Calculate the mean obliquity of the ecliptic (Earth's axial tilt)
    fn earth_axial_tilt(&self, t: f64) -> f64 {
        23.43929111 - 0.0130042 * t - 0.00000164 * t.powi(2) - 0.000000503 * t.powi(3)
    }

    /// Calculate solar ecliptic longitude using simplified VSOP87 model
    fn solar_ecliptic_longitude(t: f64) -> f64 {
        let l0 = (280.4664567 + 36000.7698278 * t + 0.0003032028 * t.powi(2)) % 360.0;
        let l1 = 1.914602 + 0.004817 * t + 0.000014 * t.powi(2);
        let l2 = 0.019993 - 0.000101 * t;
        let l3 = 0.000289;

        let m = (357.5291092 + 35999.0502909 * t).to_radians();
        l0 + l1 * m.sin() + l2 * (2.0 * m).sin() + l3 * (3.0 * m).sin()
    }

    /// Calculate the solar declination angle
    fn solar_declination(&self) -> f64 {
        let t = self.julian_century_offset();
        let epsilon = self.earth_axial_tilt(t);
        let lambda = Self::solar_ecliptic_longitude(t);
        let delta = epsilon.to_radians().sin() * lambda.to_radians().sin();
        delta.asin().to_degrees()
    }

    /// Calculate the equation of time correction
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
    fn hour_angle(&self) -> f64 {
        let hours = self.decimal_hours();
        let equation_of_time = self.solar_time_correction();
        let local_mean_time =
            hours + (self.longitude / constants::EARTH_ROTATION_RATE) + equation_of_time;

        let raw_angle = constants::EARTH_ROTATION_RATE * (local_mean_time - 12.0);
        (raw_angle % 360.0 + 360.0) % 360.0
    }

    /// Calculate atmospheric refraction correction
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
    pub(crate) fn altitude(&self) -> f64 {
        let latitude_rad = self.latitude.to_radians();
        let declination_rad = self.solar_declination().to_radians();
        let hour_angle_rad = self.hour_angle().to_radians();

        let sine_altitude = latitude_rad.sin() * declination_rad.sin()
            + latitude_rad.cos() * declination_rad.cos() * hour_angle_rad.cos();

        let true_altitude = sine_altitude.asin().to_degrees();
        true_altitude + self.atmospheric_refraction(true_altitude)
    }

    /// Calculate the sun's azimuth angle
    pub(crate) fn azimuth(&self) -> f64 {
        let latitude_rad = self.latitude.to_radians();
        let declination_rad = self.solar_declination().to_radians();
        let hour_angle_rad = self.hour_angle().to_radians();

        let sin_omega = hour_angle_rad.sin();
        let cos_omega = hour_angle_rad.cos();
        let sin_phi = latitude_rad.sin();
        let cos_phi = latitude_rad.cos();
        let tan_delta = declination_rad.tan();

        let denominator = cos_omega * sin_phi - tan_delta * cos_phi;

        // Special handling for polar regions
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
    use crate::utils::datetime::Month;

    use super::*;

    fn create_datetime(
        year: u16,
        month: Month,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> UtcDateTime {
        UtcDateTime::new(year, month, day, hour, minute, second).unwrap()
    }

    fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
        assert!(
            (a - b).abs() < epsilon,
            "assertion failed: `(left ≈ right)`\n    left: `{a}`\n   right: `{b}`\n epsilon: `{epsilon}`"
        );
    }

    #[test]
    fn test_solar_declination() {
        let sun_pos = SunPosition::new(0.0, 0.0, create_datetime(2023, Month::June, 21, 12, 0, 0));
        assert_approx_eq(sun_pos.solar_declination(), 23.45, 1.0);

        let sun_pos = SunPosition::new(
            0.0,
            0.0,
            create_datetime(2023, Month::December, 21, 12, 0, 0),
        );
        assert_approx_eq(sun_pos.solar_declination(), -23.45, 1.0);

        let sun_pos = SunPosition::new(0.0, 0.0, create_datetime(2023, Month::March, 21, 12, 0, 0));
        assert_approx_eq(sun_pos.solar_declination(), 0.0, 1.0);
    }

    #[test]
    fn test_altitude() {
        let sun_pos = SunPosition::new(0.0, 0.0, create_datetime(2023, Month::March, 21, 12, 0, 0));
        assert_approx_eq(sun_pos.altitude(), 90.0, 5.0);

        let sun_pos = SunPosition::new(90.0, 0.0, create_datetime(2023, Month::June, 21, 12, 0, 0));
        assert_approx_eq(sun_pos.altitude(), 23.45, 5.0);
    }
}
