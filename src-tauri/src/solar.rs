use std::f64::consts::PI;

use serde::{Deserialize, Serialize};
use time::{format_description, macros::offset, OffsetDateTime, Time};

#[derive(Debug, Serialize, Deserialize)]
pub struct SolarAngle {
    index: u32,
    altitude: f64,
    azimuth: f64,
}

struct SunPosition {
    latitude: f64,
    longitude: f64,
    date_time: OffsetDateTime,
    timezone_offset_hours: i8,
}

impl SunPosition {
    fn new(
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
    fn altitude(&self) -> f64 {
        let latitude_rad = Self::degrees_to_radians(self.latitude);
        let declination_rad = Self::degrees_to_radians(self.solar_declination());
        let hour_angle_rad = Self::degrees_to_radians(self.hour_angle());

        let sine_elevation = latitude_rad.sin() * declination_rad.sin()
            + latitude_rad.cos() * declination_rad.cos() * hour_angle_rad.cos();

        Self::radians_to_degrees(sine_elevation.asin())
    }

    /// Calculate the sun's azimuth angle
    fn azimuth(&self) -> f64 {
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

/// Calculate the smallest difference between two angles considering the 360-degree cycle
fn angle_difference(angle1: f64, angle2: f64) -> f64 {
    let diff = (angle1 - angle2).abs();
    if diff > 180.0 {
        360.0 - diff
    } else {
        diff
    }
}

/// Calculate the overall difference between two solar angle configurations
fn calculate_angle_difference(config: &SolarAngle, elevation: f64, azimuth: f64) -> f64 {
    let azimuth_weight = 1.0;
    let elevation_weight = 2.0;

    let azimuth_diff = angle_difference(config.azimuth, azimuth);
    let elevation_diff = (config.altitude - elevation).abs();

    azimuth_diff * azimuth_weight + elevation_diff * elevation_weight
}

/// Find the closest image configuration based on the calculated solar angles.
///
/// # Parameters
/// - `configs`: A slice of `SolarAngle` structs representing different solar angle configurations.
/// - `latitude`: Latitude of the location in decimal degrees.
/// - `longitude`: Longitude of the location in decimal degrees.
/// - `date_time`: The date and time in UTC for which to calculate the solar angles.
/// - `timezone_offset_hours`: The timezone offset in hours from UTC.
///
/// # Returns
/// An `Option<u32>` containing the index of the closest `SolarAngle` configuration, or `None` if no configurations are provided.
pub fn find_closest_image(
    configs: &[SolarAngle],
    latitude: f64,
    longitude: f64,
    date_time: OffsetDateTime,
    timezone_offset_hours: i8,
) -> Option<u32> {
    let sun_position = SunPosition::new(latitude, longitude, date_time, timezone_offset_hours);
    let elevation = sun_position.altitude();
    let azimuth = sun_position.azimuth();

    println!(
        "Calculated solar angles - Elevation: {:.1}°, Azimuth: {:.1}°",
        elevation, azimuth
    );

    configs
        .iter()
        .map(|config| {
            let difference = calculate_angle_difference(config, elevation, azimuth);
            (config.index, difference)
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(index, _)| index)
}
