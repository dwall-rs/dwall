use std::thread::sleep;
use std::time::Duration;

use time::OffsetDateTime;

use crate::{
    DwallResult,
    config::{MonitorSpecificWallpapers, WallpaperMode},
    domain::{
        geography::provider::GeographicPositionProvider,
        visual::theme::{
            engine::ThemeEngine,
            random_selector::{DailyRandomThemeSelector, filter_themes_by_pool},
        },
    },
    error::DwallError,
    infrastructure::{
        filesystem::{config_reader::ConfigReader, config_watcher::ConfigWatcher},
        platform::{
            ColorSchemeScheduler, Positioner,
            windows::display::{
                monitor_manager::DisplayMonitorProvider, wallpaper_setter::WallpaperSetter,
            },
        },
    },
    lazy::DWALL_CONFIG_DIR,
};

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

            // Handle wallpaper mode
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
                &position_provider,
            );

            info!(
                update_interval_seconds = config.interval(),
                "Starting daemon with config change detection"
            );

            self.run_engine_loop(&theme_engine, &mut consecutive_failure_count)?;
        }
    }

    /// Handle Random wallpaper mode: select theme if needed
    fn handle_random_mode(&mut self, config: &mut crate::config::Config) -> DwallResult<()> {
        let today = OffsetDateTime::now_local()?.date();

        // Check if we need random state and if we need to switch
        let needs_switch = self
            .random_state
            .as_ref()
            .map(|s| s.selector.needs_switch(&today))
            .unwrap_or(true);

        if needs_switch {
            // Get available themes from themes directory
            let available = self.list_available_themes(config)?;

            if !available.is_empty() {
                // Get user-specified pool if any
                let pool = match config.wallpaper_mode() {
                    WallpaperMode::Random { pool } => pool.as_ref(),
                    _ => None,
                };

                // Filter by pool if specified
                let themes_to_select = filter_themes_by_pool(&available, pool);

                if !themes_to_select.is_empty() {
                    // Initialize state if needed
                    let state = self.random_state.get_or_insert_with(|| RandomModeState {
                        selector: DailyRandomThemeSelector::new(),
                        current_theme: None,
                    });

                    if let Some(theme_id) = state.selector.select_next(&themes_to_select) {
                        state.selector.mark_applied(&today);
                        state.current_theme = Some(theme_id.clone());

                        // Update config: all monitors use the same theme
                        config.set_wallpaper_mode(WallpaperMode::Fixed {
                            monitor_specific_wallpapers: MonitorSpecificWallpapers::All(
                                theme_id.clone(),
                            ),
                        });

                        info!(theme_id = %theme_id, "Random theme selected for today");
                    }
                }
            }
        }

        Ok(())
    }

    /// List available themes from the themes directory
    fn list_available_themes(&self, config: &crate::config::Config) -> DwallResult<Vec<String>> {
        let themes_dir = config.themes_directory();
        let mut themes = Vec::new();

        if let Ok(entries) = std::fs::read_dir(themes_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_dir()
                    && let Some(name) = entry.file_name().to_str()
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
        theme_engine: &ThemeEngine<
            '_,
            WallpaperSetter,
            DisplayMonitorProvider,
            ColorSchemeScheduler,
            GeographicPositionProvider<'_, Positioner>,
        >,
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
