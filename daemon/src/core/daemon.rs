use crate::{
    domain::visual::theme_processor::ThemeProcessor,
    infrastructure::filesystem::config_manager::ConfigManager,
    utils::logging::setup_logging,
    DwallResult,
};

/// Main daemon application
pub struct DaemonApplication {
    config_manager: ConfigManager,
}

impl DaemonApplication {
    /// Creates a new daemon application instance
    pub async fn new() -> DwallResult<Self> {
        setup_logging(&[env!("CARGO_PKG_NAME").replace("-", "_")]);
        
        let config_manager = ConfigManager::new().await?;
        
        Ok(Self {
            config_manager,
        })
    }

    /// Runs the daemon application
    pub async fn run(&mut self) -> DwallResult<()> {
        let config = self.config_manager.read_config().await?;
        
        let theme_processor = ThemeProcessor::new(&config)?;
        theme_processor.start_update_loop().await
    }
}