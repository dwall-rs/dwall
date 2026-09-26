//! Daemon process: watches the config, runs theme update cycles and handles
//! random wallpaper mode.

use std::thread::sleep;
use std::time::Duration;

use time::{Date, OffsetDateTime};

use crate::DwallResult;
use crate::config::{
    Config, ConfigReader, ConfigWatcher, MonitorSpecificWallpapers, WallpaperMode,
};
use crate::error::DwallError;
use crate::lazy::DWALL_CONFIG_DIR;
use crate::platform::{ColorSchemeScheduler, DisplayMonitorProvider, Positioner, WallpaperSetter};
use crate::solar::GeographicPositionProvider;
use crate::theme::{DailyRandomThemeSelector, filter_themes_by_pool, is_theme_directory};
use crate::theme_engine::ThemeEngine;

const MAX_CONSECUTIVE_FAILURE_THRESHOLD: u8 = 3;

/// Runtime state for Random wallpaper mode
struct RandomModeState {
    selector: DailyRandomThemeSelector,
    current_theme: Option<String>,
}

/// Main daemon application
pub struct DaemonApplication {
    config_watcher: ConfigWatcher,
    random_state: Option<RandomModeState>,
}

impl DaemonApplication {
    /// Creates a new daemon application instance
    pub fn new() -> Self {
        let config_path = DWALL_CONFIG_DIR.join("config.toml");
        let config_watcher = ConfigWatcher::new(config_path);

        Self {
            config_watcher,
            random_state: None,
        }
    }

    /// Runs the daemon application
    pub fn run(&mut self) -> DwallResult<()> {
        let mut consecutive_failure_count = 0;

        loop {
            let mut config = ConfigReader::read_from_path(self.config_watcher.config_path())?;

            match config.wallpaper_mode() {
                WallpaperMode::Random { .. } => {
                    self.handle_random_mode(&mut config)?;
                }
                WallpaperMode::Fixed { .. } => {
                    // Fixed mode: clear random state if switching from random
                    self.random_state = None;
                }
            }

            let wallpaper_setter = WallpaperSetter::new().map_err(DwallError::WallpaperManager)?;
            let monitor_provider = DisplayMonitorProvider::new();
            let color_scheme_backend = ColorSchemeScheduler::new();
            let position_provider =
                GeographicPositionProvider::new(config.position_source(), Positioner::new());
            let theme_engine = ThemeEngine::new(
                &config,
                wallpaper_setter,
                monitor_provider,
                color_scheme_backend,
                position_provider,
            );

            info!(
                update_interval_seconds = config.interval(),
                "Starting daemon with config change detection"
            );

            self.run_engine_loop(&theme_engine, &mut consecutive_failure_count)?;
        }
    }

    /// Handle Random wallpaper mode: select today's theme and always carry it in
    /// the (in-memory) config so the engine can apply it.
    fn handle_random_mode(&mut self, config: &mut Config) -> DwallResult<()> {
        let today = OffsetDateTime::now_local()?.date();

        let needs_switch = self
            .random_state
            .as_ref()
            .map(|state| state.selector.needs_switch(&today))
            .unwrap_or(true);

        if needs_switch {
            self.select_random_theme(config, &today)?;
        }

        // Always re-apply today's selection. A same-day config reload (e.g. saving
        // any setting) re-reads `Random` from disk; without this, the engine would
        // see no `monitor_specific_wallpapers` and stop applying anything.
        if let Some(theme_id) = self
            .random_state
            .as_ref()
            .and_then(|state| state.current_theme.clone())
        {
            config.set_wallpaper_mode(WallpaperMode::Fixed {
                monitor_specific_wallpapers: MonitorSpecificWallpapers::All(theme_id),
            });
        }

        Ok(())
    }

    /// Draw today's theme from the candidate pool (once per day).
    fn select_random_theme(&mut self, config: &Config, today: &Date) -> DwallResult<()> {
        let available = self.list_available_themes(config)?;
        if available.is_empty() {
            warn!("Random mode: no themes available to select");
            return Ok(());
        }

        let pool = match config.wallpaper_mode() {
            WallpaperMode::Random { pool } => pool.as_ref(),
            _ => None,
        };

        let themes_to_select = filter_themes_by_pool(&available, pool);
        if themes_to_select.is_empty() {
            warn!("Random mode: candidate pool is empty; nothing to apply");
            return Ok(());
        }

        let state = self.random_state.get_or_insert_with(|| RandomModeState {
            selector: DailyRandomThemeSelector::new(),
            current_theme: None,
        });

        if let Some(theme_id) = state.selector.select_next(&themes_to_select) {
            state.selector.mark_applied(today);
            state.current_theme = Some(theme_id.clone());
            info!(theme_id = %theme_id, "Random theme selected for today");
        }

        Ok(())
    }

    /// List installed themes (bundled + custom) that look like valid themes.
    fn list_available_themes(&self, config: &Config) -> DwallResult<Vec<String>> {
        let mut themes: Vec<String> = Vec::new();

        for directory in [
            config.themes_directory(),
            config.customized_themes_directory(),
        ] {
            let Ok(entries) = std::fs::read_dir(directory) else {
                continue;
            };

            for entry in entries.flatten() {
                if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                    continue;
                }

                let path = entry.path();
                if !is_theme_directory(&path) {
                    continue;
                }

                if let Some(name) = entry.file_name().to_str()
                    && !themes.iter().any(|existing| existing == name)
                {
                    themes.push(name.to_string());
                }
            }
        }

        themes.sort();
        Ok(themes)
    }

    /// Runs the engine loop until config changes or max failures reached
    fn run_engine_loop(
        &mut self,
        theme_engine: &ThemeEngine<'_>,
        consecutive_failure_count: &mut u8,
    ) -> DwallResult<()> {
        let update_interval = Duration::from_secs(theme_engine.update_interval().into());

        loop {
            if self.config_watcher.has_changed()? {
                info!("Configuration file change detected, reloading configuration");
                self.config_watcher.update_modified_time()?;
                return Ok(());
            }

            match theme_engine.run_once() {
                Ok(_) => {
                    *consecutive_failure_count = 0;
                }
                Err(_) => {
                    *consecutive_failure_count += 1;
                    if *consecutive_failure_count >= MAX_CONSECUTIVE_FAILURE_THRESHOLD {
                        error!(
                            "Maximum consecutive failures reached, terminating daemon: consecutive_failures={consecutive_failure_count}"
                        );
                        std::process::exit(1);
                    }
                }
            }

            theme_engine.reload_if_monitors_changed();

            sleep(update_interval);
        }
    }
}

impl Default for DaemonApplication {
    fn default() -> Self {
        Self::new()
    }
}
