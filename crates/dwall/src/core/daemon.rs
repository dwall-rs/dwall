use std::thread::sleep;
use std::time::Duration;

use crate::{
    DwallResult,
    domain::visual::theme::engine::ThemeEngine,
    infrastructure::{
        display::wallpaper_setter::WallpaperSetter,
        filesystem::{config_reader::ConfigReader, config_watcher::ConfigWatcher},
        platform::ColorSchemeScheduler,
    },
    lazy::DWALL_CONFIG_DIR,
};

const MAX_CONSECUTIVE_FAILURE_THRESHOLD: u8 = 3;

/// Main daemon application
pub struct DaemonApplication {
    config_watcher: ConfigWatcher,
}

impl DaemonApplication {
    /// Creates a new daemon application instance
    pub fn new() -> Self {
        let config_path = DWALL_CONFIG_DIR.join("config.toml");
        let config_watcher = ConfigWatcher::new(config_path);

        Self { config_watcher }
    }

    /// Runs the daemon application
    pub fn run(&mut self) -> DwallResult<()> {
        let mut consecutive_failure_count = 0;

        loop {
            let config = ConfigReader::read_from_path(self.config_watcher.config_path())?;
            let wallpaper_setter =
                WallpaperSetter::new().map_err(crate::error::DwallError::WallpaperManager)?;
            let color_scheme_backend = ColorSchemeScheduler::new();
            let theme_engine = ThemeEngine::new(&config, wallpaper_setter, color_scheme_backend);

            info!(
                update_interval_seconds = config.interval(),
                "Starting daemon with config change detection"
            );

            self.run_engine_loop(&theme_engine, &mut consecutive_failure_count)?;
        }
    }

    /// Runs the engine loop until config changes or max failures reached
    fn run_engine_loop(
        &mut self,
        theme_engine: &ThemeEngine<'_, WallpaperSetter, ColorSchemeScheduler>,
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
