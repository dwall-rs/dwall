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
        Self::find_closest_entry(solar_configs, current_altitude, current_azimuth)
            .map(|entry| solar_configs[entry].index())
    }

    /// Finds the position of the solar-angle entry closest to the given position.
    ///
    /// A theme may reuse one image for several sun positions, so callers that
    /// need to highlight the matched point (rather than the image) use this.
    pub fn find_closest_entry(
        solar_configs: &[SolarAngle],
        current_altitude: f64,
        current_azimuth: f64,
    ) -> Option<usize> {
        solar_configs
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                // `total_cmp` instead of `partial_cmp().unwrap()`: a theme file is
                // external data and TOML allows `nan`, which made the unwrap panic
                // (and the daemon abort on the next cycle).
                solar_distance(a.altitude(), a.azimuth(), current_altitude, current_azimuth)
                    .total_cmp(&solar_distance(
                        b.altitude(),
                        b.azimuth(),
                        current_altitude,
                        current_azimuth,
                    ))
            })
            .map(|(entry, _)| entry)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn theme_angles() -> Vec<SolarAngle> {
        vec![
            SolarAngle::new(0, -18.0, 0.0),
            SolarAngle::new(1, -6.0, 90.0),
            SolarAngle::new(2, 20.0, 180.0),
            SolarAngle::new(3, 60.0, 270.0),
        ]
    }

    #[test]
    fn finds_the_nearest_entry() {
        let angles = theme_angles();

        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, -17.0, 2.0),
            Some(0)
        );
        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 21.0, 179.0),
            Some(2)
        );
        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 59.0, 269.0),
            Some(3)
        );
    }

    #[test]
    fn azimuth_wraps_around_zero() {
        let angles = vec![SolarAngle::new(0, 0.0, 5.0), SolarAngle::new(1, 0.0, 355.0)];

        // 358° is much closer to 355° than to 5° once wrapping is taken into account.
        assert_eq!(
            WallpaperSelector::find_closest_entry(&angles, 0.0, 358.0),
            Some(1)
        );
    }

    #[test]
    fn nan_angles_do_not_panic() {
        // TOML accepts `altitude = nan`; a theme file must not be able to abort the
        // daemon through the distance comparison.
        let angles = vec![
            SolarAngle::new(0, f64::NAN, 0.0),
            SolarAngle::new(1, 10.0, 20.0),
        ];

        assert!(WallpaperSelector::find_closest_entry(&angles, 11.0, 21.0).is_some());
        assert_eq!(
            WallpaperSelector::find_closest_image(&angles, 11.0, 21.0),
            Some(1)
        );
    }

    #[test]
    fn empty_theme_has_no_match() {
        assert_eq!(WallpaperSelector::find_closest_entry(&[], 0.0, 0.0), None);
    }
}
