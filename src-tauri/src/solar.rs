use std::f64::consts::PI;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct SolarAngle {
    pub index: u8,
    altitude: f64,
    azimuth: f64,
}

pub struct SunPosition {
    latitude: f64,
    longitude: f64,
    date_time: OffsetDateTime,
    timezone_offset_hours: i8,
}

impl SunPosition {
    pub fn new(
        latitude: f64,
        longitude: f64,
        date_time: OffsetDateTime,
        timezone_offset_hours: i8,
    ) -> Self {
        Self {
            latitude,
            longitude,
            date_time,
            timezone_offset_hours,
        }
    }

    fn degrees_to_radians(degrees: f64) -> f64 {
        degrees * PI / 180.0
    }

    fn radians_to_degrees(radians: f64) -> f64 {
        radians * 180.0 / PI
    }

    fn day_of_year(&self) -> u32 {
        self.date_time.ordinal() as u32
    }

    /// Calculate the solar declination angle
    fn solar_declination(&self) -> f64 {
        let day_of_year = self.day_of_year() as f64;
        // Using a more precise formula for solar declination
        let angle = 2.0 * PI * (day_of_year + 10.0) / 365.0;

        -23.45 * (angle).cos()
    }

    /// Calculate the hour angle
    fn hour_angle(&self) -> f64 {
        let hours = self.date_time.hour() as f64
            + self.date_time.minute() as f64 / 60.0
            + self.date_time.second() as f64 / 3600.0;

        // Calculate equation of time correction
        let day_of_year = self.day_of_year() as f64;
        let b = 2.0 * PI * (day_of_year - 81.0) / 364.0;
        let equation_of_time = 9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin(); // Equation of time

        // Calculate local mean time
        let zone_correction = self.longitude - (15.0 * self.timezone_offset_hours as f64); // Using provided timezone offset
        let local_mean_time = hours + (zone_correction / 15.0) + (equation_of_time / 60.0);

        // Hour angle (15 degrees per hour)
        15.0 * (local_mean_time - 12.0)
    }

    /// Calculate the sun's altitude angle (elevation)
    pub fn altitude(&self) -> f64 {
        let latitude_rad = Self::degrees_to_radians(self.latitude);
        let declination_rad = Self::degrees_to_radians(self.solar_declination());
        let hour_angle_rad = Self::degrees_to_radians(self.hour_angle());

        let sine_elevation = latitude_rad.sin() * declination_rad.sin()
            + latitude_rad.cos() * declination_rad.cos() * hour_angle_rad.cos();

        Self::radians_to_degrees(sine_elevation.asin())
    }

    /// Calculate the sun's azimuth angle
    pub fn azimuth(&self) -> f64 {
        let latitude_rad = Self::degrees_to_radians(self.latitude);
        let declination_rad = Self::degrees_to_radians(self.solar_declination());
        let hour_angle_rad = Self::degrees_to_radians(self.hour_angle());
        let elevation_rad = Self::degrees_to_radians(self.altitude());

        let cosine_azimuth = (declination_rad.sin() - latitude_rad.sin() * elevation_rad.sin())
            / (latitude_rad.cos() * elevation_rad.cos());

        let mut azimuth = Self::radians_to_degrees(cosine_azimuth.acos());

        if hour_angle_rad.sin() > 0.0 {
            azimuth = 360.0 - azimuth;
        }

        azimuth
    }
}

/// Calculate the overall difference between two solar angle configurations
// FIXME: The current angle difference algorithm is not accurate and needs further improvement.
pub fn calculate_angle_difference(
    config: &SolarAngle,
    elevation: f64,
    azimuth: f64,
    latitude: f64,
    datetime: OffsetDateTime,
) -> f64 {
    // 1. Seasonal weight factor (non-linear)
    let seasonal_factor = match datetime.month() as u8 {
        12 | 1 | 2 => 0.2, // Winter
        3..=5 => 0.4,      // Spring
        6..=8 => 0.8,      // Summer
        9..=11 => 0.5,     // Autumn
        _ => 0.5,
    };

    // 2. Latitude weight factor (using sine function for smoother transition)
    let latitude_factor = (latitude.abs() / 90.0 * std::f64::consts::PI / 2.0).sin();

    // 3. Time weight factor (modeled using a Gaussian function)
    let time_factor = {
        let hour = datetime.hour() as f64 + datetime.minute() as f64 / 60.0;
        let noon_deviation = (hour - 12.0).abs();
        let gaussian_width = 3.0; // Adjustable Gaussian width
        (-0.5 * (noon_deviation / gaussian_width).powi(2)).exp()
    };

    // 4. Independent difference calculation for elevation and azimuth
    let elevation_diff = (config.altitude - elevation).abs();
    let azimuth_diff = {
        let diff = (config.azimuth - azimuth).abs();
        diff.min(360.0 - diff)
    };

    // 5. Weights for elevation and azimuth (can be adjusted based on actual conditions)
    let elevation_weight = 0.6;
    let azimuth_weight = 0.4;

    // 6. Combined angle difference and weights
    let weighted_difference = elevation_diff * elevation_weight + azimuth_diff * azimuth_weight;

    // 7. Comprehensive weight calculation
    let comprehensive_weight = seasonal_factor * latitude_factor * time_factor;

    // 8. Final difference calculation
    weighted_difference * (1.0 - comprehensive_weight)
}
