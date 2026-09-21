//! Wallpaper selection and application abstraction.

use std::path::Path;

use crate::{DwallResult, SolarAngle};

/// Trait for setting wallpapers on monitors and lock screen.
///
/// Implemented by the platform wallpaper backend (Windows IDesktopWallpaper).
pub trait WallpaperProvider {
    /// Sets the wallpaper for a specific monitor
    fn set_monitor_wallpaper(&self, monitor_id: &str, image_path: &Path) -> DwallResult<()>;

    /// Sets the lock screen image
    fn set_lock_screen_image(&self, image_path: &Path) -> DwallResult<()>;
}

/// Selects the theme image whose solar angle is closest to the current one.
pub struct WallpaperSelector;

impl WallpaperSelector {
    /// Finds the closest matching image index based on solar angles
    pub fn find_closest_image(
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

/// Calculates the normalized distance between two solar positions.
///
/// Altitude spans [-90°, +90°] (180°) and azimuth spans [0°, 360°) and wraps.
/// Both are normalized to the same scale to avoid over-weighting azimuth.
fn solar_distance(alt1: f64, az1: f64, alt2: f64, az2: f64) -> f64 {
    let da = (alt1 - alt2) / 180.0;

    let daz_raw = (az1 - az2).abs() % 360.0;
    let daz = daz_raw.min(360.0 - daz_raw) / 360.0;

    da * da + daz * daz
}
