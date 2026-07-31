//! Wallpaper application logic for selecting and applying wallpapers

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    DisplayMonitor, DwallResult,
    config::ImageFormat,
    domain::{
        time::solar_calculator::SolarPosition,
        visual::{
            MonitorProvider, ThemeError, WallpaperProvider, theme::cache::load_cached_solar_angles,
        },
    },
};

/// Applies wallpapers to monitors based on solar position
pub(crate) struct WallpaperApplier<T: WallpaperProvider + MonitorProvider> {
    wallpaper_provider: T,
}

impl<T: WallpaperProvider + MonitorProvider> WallpaperApplier<T> {
    /// Creates a new WallpaperApplier with the given wallpaper setter
    pub(crate) fn new(wallpaper_provider: T) -> Self {
        Self { wallpaper_provider }
    }

    /// Returns a reference to the underlying wallpaper setter
    pub(crate) fn setter(&self) -> &T {
        &self.wallpaper_provider
    }

    /// Updates wallpaper for a specific monitor based on solar position
    pub(crate) fn update_monitor_wallpaper(
        &self,
        image_format: &ImageFormat,
        monitor_identifier: &str,
        theme_identifier: &str,
        theme_directory_path: &Path,
        current_sun_position: &SolarPosition,
        is_customized: bool,
    ) -> DwallResult<()> {
        let (optimal_image_index, _) =
            find_optimal_solar_wallpaper(theme_directory_path, current_sun_position)?;
        let wallpaper_file_path = construct_wallpaper_file_path(
            theme_directory_path,
            image_format.as_str(),
            optimal_image_index,
            is_customized,
        );

        info!(
            wallpaper_path = %wallpaper_file_path.display(),
            image_index = optimal_image_index,
            monitor_id = monitor_identifier,
            theme_id = theme_identifier,
            is_customized = is_customized,
            "Selected optimal solar wallpaper for monitor"
        );

        if !wallpaper_file_path.exists() {
            error!(
                wallpaper_path = %wallpaper_file_path.display(),
                theme_id = theme_identifier,
                "Selected wallpaper image file does not exist"
            );
            return Err(ThemeError::WallpaperImageMissing {
                path: wallpaper_file_path.display().to_string(),
            }
            .into());
        }

        self.wallpaper_provider
            .set_monitor_wallpaper(monitor_identifier, &wallpaper_file_path)?;

        debug!(
            monitor_id = monitor_identifier,
            wallpaper_path = %wallpaper_file_path.display(),
            "Successfully applied solar wallpaper to monitor"
        );

        Ok(())
    }

    /// Applies wallpaper to lock screen
    pub(crate) fn apply_lock_screen_wallpaper(
        &self,
        image_format: &ImageFormat,
        theme_identifier: &str,
        theme_directory_path: &Path,
        current_sun_position: &SolarPosition,
        is_customized: bool,
    ) -> DwallResult<()> {
        match find_optimal_solar_wallpaper(theme_directory_path, current_sun_position) {
            Ok((optimal_image_index, _)) => {
                let wallpaper_file_path = construct_wallpaper_file_path(
                    theme_directory_path,
                    image_format.as_str(),
                    optimal_image_index,
                    is_customized,
                );

                if wallpaper_file_path.exists() {
                    info!(
                        wallpaper_path = %wallpaper_file_path.display(),
                        theme_id = theme_identifier,
                        "Applying optimal solar wallpaper to lock screen"
                    );
                    self.wallpaper_provider
                        .set_lock_screen_image(&wallpaper_file_path)?;
                } else {
                    warn!(
                        wallpaper_path = %wallpaper_file_path.display(),
                        theme_id = theme_identifier,
                        "Optimal lock screen wallpaper file does not exist"
                    );
                    return Err(ThemeError::WallpaperImageMissing {
                        path: wallpaper_file_path.display().to_string(),
                    }
                    .into());
                }
                Ok(())
            }
            Err(wallpaper_error) => {
                warn!(
                    error = %wallpaper_error,
                    theme_id = theme_identifier,
                    "Failed to find optimal solar wallpaper for lock screen"
                );
                Err(wallpaper_error)
            }
        }
    }

    /// Lists all available monitors
    pub(crate) fn list_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        self.wallpaper_provider.get_monitors()
    }
}

/// Find the wallpaper image that best matches the current solar position
fn find_optimal_solar_wallpaper(
    theme_directory_path: &Path,
    current_sun_position: &SolarPosition,
) -> DwallResult<(u8, Vec<crate::domain::time::solar_calculator::SolarAngle>)> {
    let solar_angle_configuration = load_cached_solar_angles(theme_directory_path)?;
    let sun_altitude_degrees = current_sun_position.altitude();
    let sun_azimuth_degrees = current_sun_position.azimuth();

    debug!(
        sun_altitude = sun_altitude_degrees,
        sun_azimuth = sun_azimuth_degrees,
        theme_directory = %theme_directory_path.display(),
        "Calculated current solar position for wallpaper selection"
    );

    let optimal_image_index =
        crate::infrastructure::display::wallpaper_setter::WallpaperSetter::find_closest_image(
            &solar_angle_configuration,
            sun_altitude_degrees,
            sun_azimuth_degrees,
        )
        .ok_or_else(|| {
            error!(
                theme_directory = %theme_directory_path.display(),
                sun_altitude = sun_altitude_degrees,
                sun_azimuth = sun_azimuth_degrees,
                solar_angles_available = solar_angle_configuration.len(),
                "No suitable wallpaper image found for current solar position"
            );
            ThemeError::ImageSolarConfigurationMismatch {
                expected: solar_angle_configuration.len(),
                found: 0,
            }
        })?;

    info!(
        optimal_image_index = optimal_image_index,
        sun_altitude = sun_altitude_degrees,
        sun_azimuth = sun_azimuth_degrees,
        "Selected optimal wallpaper image for current solar position"
    );

    Ok((optimal_image_index, solar_angle_configuration))
}

/// Build wallpaper file path based on theme directory, image format, and solar image index
fn construct_wallpaper_file_path(
    theme_directory_path: &Path,
    image_format: &str,
    solar_image_index: u8,
    is_customized: bool,
) -> PathBuf {
    let image_file_name = format!("{}.{}", solar_image_index + 1, image_format);
    if is_customized {
        theme_directory_path.join("images").join(image_file_name)
    } else {
        theme_directory_path
            .join(image_format)
            .join(image_file_name)
    }
}
