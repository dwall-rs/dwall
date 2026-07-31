//! Configuration file reading operations

use std::{fs, path::Path};

use crate::{
    config::{Config, RawConfig},
    error::{ConfigError, DwallResult},
};

/// Reads configuration from the filesystem
pub struct ConfigReader;

impl ConfigReader {
    /// Reads configuration from the specified path
    pub fn read_from_path(config_path: &Path) -> DwallResult<Config> {
        if !config_path.exists() {
            warn!("Config file not found, using default configuration");
            return Ok(Config::default());
        }

        debug!(path = %config_path.display(), "Reading configuration file");

        let content = fs::read_to_string(config_path)?;
        let raw_config: RawConfig = toml::from_str(&content).map_err(|e| {
            error!(error = %e, "Failed to parse configuration");
            ConfigError::Deserialization(e)
        })?;
        let config = Config::from(raw_config);

        config.validate()?;
        info!("Configuration loaded successfully");

        Ok(config)
    }
}
