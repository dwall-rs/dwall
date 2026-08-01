//! Monitor wallpaper processing logic

use std::path::PathBuf;

use crate::{
    DwallResult,
    config::Config,
    domain::{
        time::solar_calculator::SolarPosition,
        visual::{
            ThemeError, WallpaperProvider,
            monitor::MonitorProvider,
            theme::{cache::load_cached_solar_angles, utils::get_theme_directory_path},
            wallpaper::WallpaperSelector,
        },
    },
};

pub(crate) struct MonitorProcessor;

impl MonitorProcessor {
    /// Process wallpaper for all configured monitors, returns success count
    pub(crate) fn process_all<W: WallpaperProvider, M: MonitorProvider>(
        wallpaper_provider: &W,
        monitor_provider: &M,
        config: &Config,
        solar_position: &SolarPosition,
    ) -> DwallResult<usize> {
        let monitors = monitor_provider.get_monitors()?;
        let configs = config.monitor_specific_wallpapers();
        let mut success_count = 0;

        for monitor_id in monitors.keys() {
            let theme_id = match configs.get(monitor_id) {
                Some(id) => id.as_ref(),
                None => {
                    debug!(monitor_id = monitor_id, "No theme config, skipping");
                    continue;
                }
            };

            info!(
                monitor_id = monitor_id,
                theme_id = theme_id,
                "Processing wallpaper for monitor"
            );

            if Self::process_single(
                wallpaper_provider,
                config,
                solar_position,
                monitor_id,
                theme_id,
            )
            .is_ok()
            {
                success_count += 1;
            }
        }

        Ok(success_count)
    }

    fn process_single<W: WallpaperProvider>(
        wallpaper_provider: &W,
        config: &Config,
        solar_position: &SolarPosition,
        monitor_id: &str,
        theme_id: &str,
    ) -> DwallResult<()> {
        let (theme_dir, is_customized) = get_theme_directory_path(config, theme_id);

        info!(
            theme_dir = %theme_dir.display(),
            is_customized = is_customized,
            theme_id = theme_id,
            "Using theme directory"
        );

        let image_path = Self::select_wallpaper_path(
            config,
            &theme_dir,
            solar_position,
            is_customized,
            theme_id,
        )?;

        wallpaper_provider.set_monitor_wallpaper(monitor_id, &image_path)?;

        debug!(
            monitor_id = monitor_id,
            wallpaper_path = %image_path.display(),
            "Successfully applied solar wallpaper to monitor"
        );

        Ok(())
    }

    /// Process lock screen wallpaper if enabled
    pub(crate) fn process_lock_screen<W: WallpaperProvider>(
        wallpaper_provider: &W,
        config: &Config,
        solar_position: &SolarPosition,
        lock_screen_theme_id: &str,
    ) -> DwallResult<()> {
        if !config.lock_screen_wallpaper_enabled() {
            return Ok(());
        }

        let (theme_dir, is_customized) = get_theme_directory_path(config, lock_screen_theme_id);

        let image_path = Self::select_wallpaper_path(
            config,
            &theme_dir,
            solar_position,
            is_customized,
            lock_screen_theme_id,
        )?;

        wallpaper_provider.set_lock_screen_image(&image_path)?;

        Ok(())
    }

    /// Select wallpaper path based on solar position
    fn select_wallpaper_path(
        config: &Config,
        theme_dir: &std::path::Path,
        solar_position: &SolarPosition,
        is_customized: bool,
        theme_id: &str,
    ) -> DwallResult<PathBuf> {
        let solar_angle_configuration = load_cached_solar_angles(theme_dir)?;
        let sun_altitude_degrees = solar_position.altitude();
        let sun_azimuth_degrees = solar_position.azimuth();

        debug!(
            sun_altitude = sun_altitude_degrees,
            sun_azimuth = sun_azimuth_degrees,
            theme_directory = %theme_dir.display(),
            "Calculated current solar position for wallpaper selection"
        );

        let optimal_image_index = WallpaperSelector::find_closest_image(
            &solar_angle_configuration,
            sun_altitude_degrees,
            sun_azimuth_degrees,
        )
        .ok_or(ThemeError::ImageSolarConfigurationMismatch {
            expected: solar_angle_configuration.len(),
            found: 0,
        })?;

        let image_file_name = format!(
            "{}.{}",
            optimal_image_index + 1,
            config.image_format().as_str()
        );
        let image_path = if is_customized {
            theme_dir.join("images").join(image_file_name)
        } else {
            theme_dir
                .join(config.image_format().as_str())
                .join(image_file_name)
        };

        if !image_path.exists() {
            return Err(ThemeError::WallpaperImageMissing {
                path: image_path.display().to_string(),
            }
            .into());
        }

        info!(
            wallpaper_path = %image_path.display(),
            image_index = optimal_image_index,
            theme_id = theme_id,
            "Selected optimal solar wallpaper"
        );

        Ok(image_path)
    }
}
