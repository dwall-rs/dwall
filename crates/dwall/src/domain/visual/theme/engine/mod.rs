//! Theme engine that orchestrates wallpaper and color scheme updates

pub mod monitor_processor;
pub mod update_loop;

use crate::{
    DwallResult,
    config::Config,
    domain::visual::{
        WallpaperProvider,
        color_scheme::{ColorSchemeApplier, ColorSchemeProvider},
        monitor::MonitorProvider,
        solar_source::SolarPositionSource,
        wallpaper::applier::WallpaperApplier,
    },
};

/// Orchestrates the solar-based theme update cycle
pub(crate) struct ThemeEngine<'a, W: WallpaperProvider + MonitorProvider, T: ColorSchemeProvider> {
    config: &'a Config,
    solar_source: SolarPositionSource<'a>,
    wallpaper_applier: WallpaperApplier<W>,
    color_scheme_applier: ColorSchemeApplier<T>,
}

impl<'a, W: WallpaperProvider + MonitorProvider, T: ColorSchemeProvider> ThemeEngine<'a, W, T> {
    /// Creates a new ThemeEngine with the given configuration and dependencies
    pub(crate) fn new(config: &'a Config, wallpaper_setter: W, color_scheme_provider: T) -> Self {
        info!(
            auto_detect_color_mode = config.auto_detect_color_scheme(),
            image_format = ?config.image_format(),
            update_interval_seconds = config.interval(),
            "Initializing theme engine"
        );

        Self {
            solar_source: SolarPositionSource::new(config.position_source()),
            wallpaper_applier: WallpaperApplier::new(wallpaper_setter),
            color_scheme_applier: ColorSchemeApplier::new(color_scheme_provider),
            config,
        }
    }

    /// Returns the update interval in seconds
    pub(crate) fn update_interval(&self) -> u16 {
        self.config.interval()
    }

    /// Runs a single theme update cycle
    pub(crate) fn run_once(&self) -> DwallResult<bool> {
        let position = self.solar_source.get_current_position()?;
        self.process_theme_cycle(&position)?;
        Ok(true)
    }

    /// Checks if monitor configuration has changed and reloads if necessary
    pub(crate) fn reload_if_monitors_changed(&self) -> bool {
        let changed = self
            .wallpaper_applier
            .setter()
            .has_configuration_changed()
            .unwrap_or(false);

        if changed {
            info!("Monitor configuration changed, reapplying wallpapers");
            if let Ok(position) = self.solar_source.get_current_position()
                && let Err(e) = self.process_theme_cycle(&position)
            {
                error!(error = %e, "Failed to reapply after monitor change");
            }
        }

        changed
    }

    /// Starts a continuous loop to update themes based on current solar position
    pub fn start_update_loop(&self) -> DwallResult<()> {
        let loop_config = update_loop::LoopConfig {
            max_consecutive_failures: 3,
            update_interval: std::time::Duration::from_secs(self.config.interval().into()),
        };

        update_loop::UpdateLoop.run(
            &loop_config,
            || {
                let position = self.solar_source.get_current_position()?;
                self.process_theme_cycle(&position)
            },
            || self.reload_if_monitors_changed(),
        )
    }

    /// Process theme cycle for the current geographic position
    pub(crate) fn process_theme_cycle(
        &self,
        position: &crate::domain::geography::Position,
    ) -> DwallResult<()> {
        let solar_position = self.solar_source.get_current_solar_position()?;
        let available_monitors = self.wallpaper_applier.list_monitors()?;

        let success_count = monitor_processor::MonitorProcessor::process_all(
            &self.wallpaper_applier,
            self.config,
            &solar_position,
        )?;

        if success_count > 0
            && let Some(theme_id) = self.get_first_configured_theme()
            && let Err(e) = monitor_processor::MonitorProcessor::process_lock_screen(
                &self.wallpaper_applier,
                self.config,
                &solar_position,
                &theme_id,
            )
        {
            warn!(error = %e, theme_id = theme_id, "Failed to apply lock screen wallpaper");
        }

        self.color_scheme_applier
            .update_color_scheme(self.config, position, &solar_position)?;

        info!(
            successful_monitors = success_count,
            total_monitors = available_monitors.len(),
            "Theme cycle completed"
        );

        Ok(())
    }

    fn get_first_configured_theme(&self) -> Option<String> {
        match self.config.monitor_specific_wallpapers() {
            crate::config::MonitorSpecificWallpapers::All(theme_id) => Some(theme_id.clone()),
            crate::config::MonitorSpecificWallpapers::Individual(map) => {
                map.values().next().cloned()
            }
        }
    }
}

/// Applies a solar theme and starts background processing for periodic updates
pub async fn apply_solar_theme<W, T>(
    configuration: Config,
    wallpaper_setter: W,
    color_scheme_provider: T,
) -> DwallResult<()>
where
    W: WallpaperProvider + MonitorProvider,
    T: ColorSchemeProvider,
{
    if configuration.monitor_specific_wallpapers().is_empty() {
        warn!(
            "No monitor-specific wallpaper configurations found, theme daemon will not be started"
        );
        return Err(crate::domain::visual::ThemeError::MonitorWallpaperConfigurationMissing.into());
    }

    info!(
        configured_monitors = ?configuration.monitor_specific_wallpapers(),
        "Starting theme engine with monitor configurations"
    );

    ThemeEngine::new(&configuration, wallpaper_setter, color_scheme_provider).start_update_loop()
}
