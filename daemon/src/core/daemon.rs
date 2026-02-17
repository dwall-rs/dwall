use std::thread::sleep;
use std::time::Duration;

use crate::{
    domain::visual::theme_processor::SolarThemeProcessor,
    infrastructure::filesystem::config_manager::ConfigManager, DwallResult,
};

const MAX_CONSECUTIVE_FAILURE_THRESHOLD: u8 = 3;

/// Main daemon application
pub struct DaemonApplication {
    config_manager: ConfigManager,
}

impl DaemonApplication {
    /// Creates a new daemon application instance
    pub fn new() -> Self {
        let config_manager = ConfigManager::new();

        Self { config_manager }
    }

    /// Runs the daemon application
    pub fn run(&mut self) -> DwallResult<()> {
        let mut consecutive_failure_count = 0;

        loop {
            let config = self.config_manager.read_config()?;
            let theme_processor = SolarThemeProcessor::new(&config)?;

            info!(
                update_interval_seconds = config.interval(),
                "Starting daemon with config change detection"
            );

            self.run_processor_loop(&theme_processor, &mut consecutive_failure_count)?;
        }
    }

    /// Runs the processor loop until config changes or max failures reached
    fn run_processor_loop(
        &mut self,
        theme_processor: &SolarThemeProcessor,
        consecutive_failure_count: &mut u8,
    ) -> DwallResult<()> {
        let update_interval = Duration::from_secs(theme_processor.update_interval().into());

        loop {
            if self.config_manager.has_changed()? {
                info!("Configuration file change detected, reloading configuration");
                return Ok(());
            }

            match theme_processor.run_single_cycle() {
                Ok(_) => {
                    *consecutive_failure_count = 0;
                }
                Err(_) => {
                    *consecutive_failure_count += 1;
                    if *consecutive_failure_count >= MAX_CONSECUTIVE_FAILURE_THRESHOLD {
                        error!(
                            consecutive_failures = consecutive_failure_count,
                            "Maximum consecutive failures reached, terminating daemon"
                        );
                        std::process::exit(1);
                    }
                }
            }

            theme_processor.check_and_reload_monitor_configuration();

            sleep(update_interval);
        }
    }
}

impl Default for DaemonApplication {
    fn default() -> Self {
        Self::new()
    }
}
