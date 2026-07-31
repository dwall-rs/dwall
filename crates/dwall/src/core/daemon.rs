use std::thread::sleep;
use std::time::Duration;

use crate::{
    DwallResult,
    domain::{geography::provider::GeographicPositionProvider, visual::theme::engine::ThemeEngine},
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
