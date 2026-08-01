//! Configuration file writing operations

use std::fs;

use crate::{
    config::Config,
    error::{ConfigError, DwallResult},
};

/// Writes configuration to the filesystem
pub struct ConfigWriter;

impl ConfigWriter {
    /// Writes configuration to the specified path
    pub fn write_to_path(&self, config_path: &std::path::Path, config: &Config) -> DwallResult<()> {
        config.validate()?;

        let toml_string = toml::to_string(config).map_err(|e| {
            error!(error = %e, "Failed to serialize configuration");
            ConfigError::Serialization(e)
        })?;

        info!(path = %config_path.display(), "Writing configuration file");
        fs::write(config_path, toml_string.as_bytes())?;
        Ok(())
    }
}
