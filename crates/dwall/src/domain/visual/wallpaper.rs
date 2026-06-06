//! Wallpaper management and selection logic

use crate::domain::time::solar_calculator::SolarAngle;

/// Wallpaper selection utilities
pub struct WallpaperSelector;

impl WallpaperSelector {
    /// Finds the closest matching image based on solar angles
    pub(crate) fn find_closest_image(
        solar_configs: &[SolarAngle],
        current_altitude: f64,
        current_azimuth: f64,
    ) -> Option<u8> {
        solar_configs
            .iter()
            .min_by(|a, b| {
                solar_distance(a.altitude(), a.azimuth(), current_altitude, current_azimuth)
                    .partial_cmp(&solar_distance(
                        b.altitude(),
                        b.azimuth(),
                        current_altitude,
                        current_azimuth,
                    ))
                    .unwrap()
            })
            .map(|sa| sa.index())
    }
}

/// Calculates the normalized distance between two solar positions
///
/// altitude range `[-90°, +90°]` (span of 180°)
/// azimuth  range `[0°, 360°)` (span of 360°, and wraps around)
/// After normalization both have the same scale, avoiding overly large azimuth weight
fn solar_distance(alt1: f64, az1: f64, alt2: f64, az2: f64) -> f64 {
    let da = (alt1 - alt2) / 180.0;

    // Azimuth is a circular space; the difference between 359° and 1° is 2°, not 358°
    let daz_raw = (az1 - az2).abs() % 360.0;
    let daz = daz_raw.min(360.0 - daz_raw) / 360.0;

    da * da + daz * daz // No need to take square root; the comparison result is the same
}
