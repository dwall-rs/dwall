//! Monitor wallpaper processing logic

use crate::{
    DwallResult,
    config::Config,
    domain::{
        time::solar_calculator::SolarPosition,
        visual::{
            WallpaperProvider, monitor::MonitorProvider, wallpaper::applier::WallpaperApplier,
        },
    },
};

pub(crate) struct MonitorProcessor;

impl MonitorProcessor {
    /// Process wallpaper for all configured monitors, returns success count
    pub(crate) fn process_all<W>(
        wallpaper_applier: &WallpaperApplier<W>,
        config: &Config,
        solar_position: &SolarPosition,
    ) -> DwallResult<usize>
    where
        W: WallpaperProvider + MonitorProvider,
    {
        let monitors = wallpaper_applier.list_monitors()?;
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
                wallpaper_applier,
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

    fn process_single<W>(
        wallpaper_applier: &WallpaperApplier<W>,
        config: &Config,
        solar_position: &SolarPosition,
        monitor_id: &str,
        theme_id: &str,
    ) -> DwallResult<()>
    where
        W: WallpaperProvider + MonitorProvider,
    {
        let (theme_dir, is_customized) =
            super::super::utils::get_theme_directory_path(config, theme_id);

        info!(
            theme_dir = %theme_dir.display(),
            is_customized = is_customized,
            theme_id = theme_id,
            "Using theme directory"
        );

        wallpaper_applier.update_monitor_wallpaper(
            config.image_format(),
            monitor_id,
            theme_id,
            &theme_dir,
            solar_position,
            is_customized,
        )
    }

    /// Process lock screen wallpaper if enabled
    pub(crate) fn process_lock_screen<W>(
        wallpaper_applier: &WallpaperApplier<W>,
        config: &Config,
        solar_position: &SolarPosition,
        lock_screen_theme_id: &str,
    ) -> DwallResult<()>
    where
        W: WallpaperProvider + MonitorProvider,
    {
        if !config.lock_screen_wallpaper_enabled() {
            return Ok(());
        }

        let (theme_dir, is_customized) =
            super::super::utils::get_theme_directory_path(config, lock_screen_theme_id);

        wallpaper_applier.apply_lock_screen_wallpaper(
            config.image_format(),
            lock_screen_theme_id,
            &theme_dir,
            solar_position,
            is_customized,
        )
    }
}
