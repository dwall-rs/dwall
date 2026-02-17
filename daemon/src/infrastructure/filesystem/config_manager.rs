//! Configuration and filesystem management infrastructure

use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{
    config::Config,
    error::{ConfigError, DwallResult},
    lazy::DWALL_CONFIG_DIR,
};

/// Configuration manager for file operations
pub(crate) struct ConfigManager {
    config_path: PathBuf,
    last_modified: Option<SystemTime>,
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
            last_modified: None,
        }
    }

    /// Gets the current modification time from the filesystem
    fn get_file_modified_time(&self) -> DwallResult<Option<SystemTime>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(&self.config_path)?;
        let modified = metadata.modified()?;
        Ok(Some(modified))
    }

    /// Checks if the configuration file has changed since last read
    ///
    /// Returns `true` if:
    /// - The file didn't exist before but now exists
    /// - The file existed before but now doesn't exist
    /// - The file's modification time is different from the last known time
    pub(crate) fn has_changed(&self) -> DwallResult<bool> {
        let current_modified = self.get_file_modified_time()?;

        let has_changed = match (self.last_modified, current_modified) {
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
            (Some(last), Some(current)) => last != current,
        };

        if has_changed {
            debug!(
                path = %self.config_path.display(),
                last_modified = ?self.last_modified,
                current_modified = ?current_modified,
                "Configuration file change detected"
            );
        }

        Ok(has_changed)
    }

    /// Reads the configuration from the file system and updates the last modified time
    pub(crate) fn read_config(&mut self) -> DwallResult<Config> {
        if !self.config_path.exists() {
            warn!("Config file not found, using default configuration");
            self.last_modified = None;
            return Ok(Config::default());
        }

        debug!(path = %self.config_path.display(), "Reading configuration file");

        let metadata = fs::metadata(&self.config_path)?;
        self.last_modified = Some(metadata.modified()?);

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
    pub(crate) fn write_config(&mut self, config: &Config) -> DwallResult<()> {
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
    let mut config_manager = ConfigManager::with_config_dir(&DWALL_CONFIG_DIR);
    config_manager.read_config()
}

/// Convenience function to write the configuration to the default location
pub fn write_config_file(config: &Config) -> DwallResult<()> {
    let mut config_manager = ConfigManager::with_config_dir(&DWALL_CONFIG_DIR);
    config_manager.write_config(config)
}
