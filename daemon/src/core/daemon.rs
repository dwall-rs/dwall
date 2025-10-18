use crate::{
    domain::visual::theme_processor::SolarThemeProcessor,
    infrastructure::filesystem::config_manager::ConfigManager, DwallResult,
};

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
        let config = self.config_manager.read_config()?;

        let theme_processor = SolarThemeProcessor::new(&config)?;
        theme_processor.start_solar_update_loop()
    }
}

impl Default for DaemonApplication {
    fn default() -> Self {
        Self::new()
    }
}
