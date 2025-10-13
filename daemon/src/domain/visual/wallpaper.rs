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
        let min_altitude_diff = solar_configs
            .iter()
            .map(|sa| (sa.altitude - current_altitude).abs())
            .min_by(|a, b| a.partial_cmp(b).unwrap())?;

        let closest_altitude_matches: Vec<&SolarAngle> = solar_configs
            .iter()
            .filter(|sa| (sa.altitude - current_altitude).abs() == min_altitude_diff)
            .collect();

        if closest_altitude_matches.len() == 1 {
            return closest_altitude_matches[0].index.into();
        }

        closest_altitude_matches
            .iter()
            .min_by(|&&a, &&b| {
                (a.azimuth - current_azimuth)
                    .abs()
                    .partial_cmp(&(b.azimuth - current_azimuth).abs())
                    .unwrap()
            })
            .map(|&sa| sa.index)
    }
}
