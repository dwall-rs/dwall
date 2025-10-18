//! Configuration and filesystem management infrastructure

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::Config,
    error::{ConfigError, DwallResult},
    lazy::DWALL_CONFIG_DIR,
};

/// Configuration manager for file operations
pub(crate) struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new ConfigManager with the default config directory
    pub(crate) fn new() -> Self {
        Self::with_config_dir(&DWALL_CONFIG_DIR)
    }

    /// Creates a new ConfigManager with a specific config directory
    pub(crate) fn with_config_dir(config_dir: &Path) -> Self {
        Self {
            config_path: config_dir.join("config.toml"),
        }
    }

    /// Reads the configuration from the file system
    pub(crate) fn read_config(&self) -> DwallResult<Config> {
        if !self.config_path.exists() {
            warn!("Config file not found, using default configuration");
            return Ok(Config::default());
        }

        debug!(path = %self.config_path.display(), "Reading configuration file");

        let content = fs::read_to_string(&self.config_path)?;
        let config: Config = toml::from_str(&content).map_err(|e| {
            error!(error = %e, "Failed to parse configuration");
            ConfigError::Deserialization(e)
        })?;

        config.validate()?;
        info!("Configuration loaded successfully");

        Ok(config)
    }

    /// Writes the configuration to the file system
    pub(crate) fn write_config(&self, config: &Config) -> DwallResult<()> {
        config.validate()?;

        if !self.config_path.exists() {
            return self.write_config_to_file(config);
        }

        if let Ok(existing_config) = self.read_config() {
            if existing_config == *config {
                debug!("Configuration unchanged, skipping write");
                return Ok(());
            }
        }

        self.write_config_to_file(config)
    }

    pub(crate) fn write_config_to_file(&self, config: &Config) -> DwallResult<()> {
        let toml_string = toml::to_string(config).map_err(|e| {
            error!(error = %e, "Failed to serialize configuration");
            ConfigError::Serialization(e)
        })?;

        info!(path = %self.config_path.display(), "Writing configuration file");
        fs::write(&self.config_path, toml_string.as_bytes())?;
        Ok(())
    }
}

/// Convenience function to read the configuration file from the default location
pub fn read_config_file() -> DwallResult<Config> {
    let config_manager = ConfigManager::with_config_dir(&DWALL_CONFIG_DIR);
    config_manager.read_config()
}

/// Convenience function to write the configuration to the default location
pub fn write_config_file(config: &Config) -> DwallResult<()> {
    let config_manager = ConfigManager::with_config_dir(&DWALL_CONFIG_DIR);
    config_manager.write_config(config)
}
