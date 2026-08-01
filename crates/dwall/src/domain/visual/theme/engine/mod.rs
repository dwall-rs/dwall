//! Theme engine that orchestrates wallpaper and color scheme updates

pub mod monitor_processor;
pub mod update_loop;

use std::time::Duration;

use time::OffsetDateTime;

use crate::{
    DwallResult, Position,
    config::{Config, MonitorSpecificWallpapers},
    domain::{
        geography::PositionProvider,
        time::solar_calculator::SolarPosition,
        visual::{
            ThemeError, WallpaperProvider,
            color_scheme::{ColorSchemeApplier, ColorSchemeProvider},
            monitor::MonitorProvider,
            theme::engine::update_loop::{LoopConfig, UpdateLoop},
            wallpaper::applier::WallpaperApplier,
        },
    },
};

/// Orchestrates the solar-based theme update cycle
pub(crate) struct ThemeEngine<'a, W, M, T, P>
where
    W: WallpaperProvider,
    M: MonitorProvider,
    T: ColorSchemeProvider,
    P: PositionProvider,
{
    config: &'a Config,
    position_provider: &'a P,
    wallpaper_applier: WallpaperApplier<W, M>,
    color_scheme_applier: ColorSchemeApplier<T>,
}

impl<'a, W, M, T, P> ThemeEngine<'a, W, M, T, P>
where
    W: WallpaperProvider,
    M: MonitorProvider,
    T: ColorSchemeProvider,
    P: PositionProvider,
{
    /// Creates a new ThemeEngine with the given configuration and dependencies
    pub(crate) fn new(
        config: &'a Config,
        wallpaper_provider: W,
        monitor_provider: M,
        color_scheme_provider: T,
        position_provider: &'a P,
    ) -> Self {
        info!(
            auto_detect_color_mode = config.auto_detect_color_scheme(),
            image_format = ?config.image_format(),
            update_interval_seconds = config.interval(),
            "Initializing theme engine"
        );

        Self {
            position_provider,
            wallpaper_applier: WallpaperApplier::new(wallpaper_provider, monitor_provider),
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
        let position = self.position_provider.get_current_position()?;
        self.process_theme_cycle(&position)?;
        Ok(true)
    }

    /// Checks if monitor configuration has changed and reloads if necessary
    pub(crate) fn reload_if_monitors_changed(&self) -> bool {
        let changed = self
            .wallpaper_applier
            .monitor()
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

    /// Starts a continuous loop to update themes based on current solar position
    pub fn start_update_loop(&self) -> DwallResult<()> {
        let loop_config = LoopConfig {
            max_consecutive_failures: 3,
            update_interval: Duration::from_secs(self.config.interval().into()),
        };

        UpdateLoop.run(
            &loop_config,
            || {
                let position = self.position_provider.get_current_position()?;

                self.process_theme_cycle(&position)
            },
            || self.reload_if_monitors_changed(),
        )
    }

    /// Process theme cycle for the current geographic position
    pub(crate) fn process_theme_cycle(&self, position: &Position) -> DwallResult<()> {
        let now_local = OffsetDateTime::now_local()?;
        let now = now_local.assume_utc();
        let solar_position = SolarPosition::new(position, &now);
        let available_monitors = self.wallpaper_applier.list_monitors()?;

        let success_count = monitor_processor::MonitorProcessor::process_all(
            self.wallpaper_applier.setter(),
            self.wallpaper_applier.monitor(),
            self.config,
            &solar_position,
        )?;

        if success_count > 0
            && let Some(theme_id) = self.get_first_configured_theme()
            && let Err(e) = monitor_processor::MonitorProcessor::process_lock_screen(
                self.wallpaper_applier.setter(),
                self.config,
                &solar_position,
                &theme_id,
            )
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

    fn get_first_configured_theme(&self) -> Option<String> {
        match self.config.monitor_specific_wallpapers() {
            Some(MonitorSpecificWallpapers::All(theme_id)) => Some(theme_id.clone()),
            Some(MonitorSpecificWallpapers::Individual(map)) => map.values().next().cloned(),
            None => None,
        }
    }
}

/// Applies a solar theme and starts background processing for periodic updates
pub async fn apply_solar_theme<'a, W, M, T, P>(
    configuration: &'a Config,
    wallpaper_provider: W,
    monitor_provider: M,
    color_scheme_provider: T,
    position_provider: &'a P,
) -> DwallResult<()>
where
    W: WallpaperProvider,
    M: MonitorProvider,
    T: ColorSchemeProvider,
    P: PositionProvider,
{
    if configuration
        .monitor_specific_wallpapers()
        .is_none_or(|w| w.is_empty())
    {
        warn!(
            "No monitor-specific wallpaper configurations found, theme daemon will not be started"
        );
        return Err(ThemeError::MonitorWallpaperConfigurationMissing.into());
    }

    info!(
        configured_monitors = ?configuration.monitor_specific_wallpapers(),
        "Starting theme engine with monitor configurations"
    );

    ThemeEngine::new(
        configuration,
        wallpaper_provider,
        monitor_provider,
        color_scheme_provider,
        position_provider,
    )
    .start_update_loop()
}
