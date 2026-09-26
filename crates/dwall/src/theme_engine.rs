//! Theme update engine: applies wallpapers and color scheme for a single cycle.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use time::OffsetDateTime;

use crate::DwallResult;
use crate::color_scheme::ColorSchemeApplier;
use crate::config::{Config, MonitorSpecificWallpapers};
use crate::monitor::MonitorProvider;
use crate::platform::{
    ColorSchemeScheduler, DisplayMonitor, DisplayMonitorProvider, Positioner, WallpaperSetter,
};
use crate::solar::{GeographicPositionProvider, Position, SolarPosition};
use crate::theme::{
    ThemeError, get_theme_directory_path, load_cached_theme_data, wallpaper_image_path,
};
use crate::wallpaper::{WallpaperProvider, WallpaperSelector};

/// Orchestrates the solar-based theme update cycle.
pub(crate) struct ThemeEngine<'a> {
    config: &'a Config,
    position_provider: GeographicPositionProvider<'a, Positioner>,
    wallpaper_setter: WallpaperSetter,
    monitor_provider: DisplayMonitorProvider,
    color_scheme_applier: ColorSchemeApplier<ColorSchemeScheduler>,
}

impl<'a> ThemeEngine<'a> {
    pub(crate) fn new(
        config: &'a Config,
        wallpaper_setter: WallpaperSetter,
        monitor_provider: DisplayMonitorProvider,
        color_scheme_provider: ColorSchemeScheduler,
        position_provider: GeographicPositionProvider<'a, Positioner>,
    ) -> Self {
        info!(
            auto_detect_color_mode = config.auto_detect_color_scheme(),
            image_format = ?config.image_format(),
            update_interval_seconds = config.interval(),
            "Initializing theme engine"
        );

        Self {
            config,
            position_provider,
            wallpaper_setter,
            monitor_provider,
            color_scheme_applier: ColorSchemeApplier::new(color_scheme_provider),
        }
    }

    /// Returns the update interval in seconds
    pub(crate) fn update_interval(&self) -> u16 {
        self.config.interval()
    }

    /// Runs a single theme update cycle
    pub(crate) fn run_once(&self) -> DwallResult<bool> {
        let position = self.position_provider.get_current_position()?;
        self.process_theme_cycle(&position)?;
        Ok(true)
    }

    /// Checks if monitor configuration has changed and reloads if necessary
    pub(crate) fn reload_if_monitors_changed(&self) -> bool {
        let changed = self
            .monitor_provider
            .has_configuration_changed()
            .unwrap_or(false);

        if changed {
            info!("Monitor configuration changed, reapplying wallpapers");
            if let Ok(position) = self.position_provider.get_current_position()
                && let Err(e) = self.process_theme_cycle(&position)
            {
                error!(error = %e, "Failed to reapply after monitor change");
            }
        }

        changed
    }

    /// Process theme cycle for the current geographic position
    pub(crate) fn process_theme_cycle(&self, position: &Position) -> DwallResult<()> {
        let now_local = OffsetDateTime::now_local()?;
        let now = now_local.assume_utc();
        let solar_position = SolarPosition::new(position, &now);
        let available_monitors = self.monitor_provider.get_monitors()?;

        let success_count = self.process_monitors(&available_monitors, &solar_position)?;

        if success_count > 0
            && let Some(theme_id) = self.get_first_configured_theme()
            && let Err(e) = self.process_lock_screen(&solar_position, &theme_id)
        {
            warn!(error = %e, theme_id = theme_id, "Failed to apply lock screen wallpaper");
        }

        self.color_scheme_applier.update_color_scheme(
            self.config,
            &now_local,
            position,
            &solar_position,
        )?;

        info!(
            successful_monitors = success_count,
            total_monitors = available_monitors.len(),
            "Theme cycle completed"
        );

        Ok(())
    }

    /// Applies the wallpaper for every configured monitor, returning the count
    /// of successes. Reuses the caller's monitor snapshot instead of re-querying.
    fn process_monitors(
        &self,
        monitors: &HashMap<String, DisplayMonitor>,
        solar_position: &SolarPosition,
    ) -> DwallResult<usize> {
        let configs = self.config.monitor_specific_wallpapers();
        let mut success_count = 0;

        for monitor_id in monitors.keys() {
            let theme_id = match configs.as_ref().and_then(|c| c.get(monitor_id)) {
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

            if self
                .process_single(monitor_id, theme_id, solar_position)
                .is_ok()
            {
                success_count += 1;
            }
        }

        Ok(success_count)
    }

    fn process_single(
        &self,
        monitor_id: &str,
        theme_id: &str,
        solar_position: &SolarPosition,
    ) -> DwallResult<()> {
        let (theme_dir, is_customized) = get_theme_directory_path(self.config, theme_id);

        info!(
            theme_dir = %theme_dir.display(),
            is_customized = is_customized,
            theme_id = theme_id,
            "Using theme directory"
        );

        let image_path =
            self.select_wallpaper_path(&theme_dir, solar_position, is_customized, theme_id)?;

        self.wallpaper_setter
            .set_monitor_wallpaper(monitor_id, &image_path)?;

        debug!(
            monitor_id = monitor_id,
            wallpaper_path = %image_path.display(),
            "Successfully applied solar wallpaper to monitor"
        );

        Ok(())
    }

    /// Applies the lock screen wallpaper if enabled
    fn process_lock_screen(
        &self,
        solar_position: &SolarPosition,
        lock_screen_theme_id: &str,
    ) -> DwallResult<()> {
        if !self.config.lock_screen_wallpaper_enabled() {
            return Ok(());
        }

        let (theme_dir, is_customized) =
            get_theme_directory_path(self.config, lock_screen_theme_id);

        let image_path = self.select_wallpaper_path(
            &theme_dir,
            solar_position,
            is_customized,
            lock_screen_theme_id,
        )?;

        self.wallpaper_setter.set_lock_screen_image(&image_path)?;

        Ok(())
    }

    /// Selects the wallpaper image path matching the current solar position
    fn select_wallpaper_path(
        &self,
        theme_dir: &Path,
        solar_position: &SolarPosition,
        is_customized: bool,
        theme_id: &str,
    ) -> DwallResult<PathBuf> {
        let theme_data = load_cached_theme_data(theme_dir)?;
        let sun_altitude_degrees = solar_position.altitude();
        let sun_azimuth_degrees = solar_position.azimuth();

        debug!(
            sun_altitude = sun_altitude_degrees,
            sun_azimuth = sun_azimuth_degrees,
            theme_directory = %theme_dir.display(),
            "Calculated current solar position for wallpaper selection"
        );

        let optimal_image_index = WallpaperSelector::find_closest_image(
            theme_data.angles.as_slice(),
            sun_altitude_degrees,
            sun_azimuth_degrees,
        )
        .ok_or(ThemeError::ImageSolarConfigurationMismatch {
            expected: theme_data.angles.len(),
            found: 0,
        })?;

        // Custom themes declare their own image format; legacy themes use the config's.
        let image_format = theme_data
            .image_format
            .clone()
            .unwrap_or_else(|| self.config.image_format().clone());

        let image_path =
            wallpaper_image_path(theme_dir, optimal_image_index, &image_format, is_customized);

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

    fn get_first_configured_theme(&self) -> Option<String> {
        match self.config.monitor_specific_wallpapers() {
            Some(MonitorSpecificWallpapers::All(theme_id)) => Some(theme_id.clone()),
            Some(MonitorSpecificWallpapers::Individual(map)) => map.values().next().cloned(),
            None => None,
        }
    }
}
