use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use thiserror::Error;

use crate::{error::DwallResult, lazy::DWALL_CONFIG_DIR};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("Configuration validation failed")]
    Validation,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    #[default]
    Jpeg,
}

impl From<&ImageFormat> for &str {
    fn from(val: &ImageFormat) -> Self {
        match val {
            ImageFormat::Jpeg => "jpg",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "UPPERCASE", tag = "type")]
pub enum CoordinateSource {
    Automatic {
        #[serde(default = "default_update_on_each_calculation")]
        update_on_each_calculation: bool,
    },

    Manual {
        latitude: f64,
        longitude: f64,
    },
}

impl Default for CoordinateSource {
    fn default() -> Self {
        Self::Automatic {
            update_on_each_calculation: false,
        }
    }
}

fn default_update_on_each_calculation() -> bool {
    false
}

impl CoordinateSource {
    pub fn validate(&self) -> bool {
        match *self {
            CoordinateSource::Automatic { .. } => true,
            CoordinateSource::Manual {
                latitude,
                longitude,
            } => (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum MonitorSpecificWallpapers {
    All(String),
    Specific(HashMap<String, String>),
}

impl MonitorSpecificWallpapers {
    pub fn is_all(&self) -> bool {
        matches!(self, MonitorSpecificWallpapers::All(_))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MonitorSpecificWallpapers::All(_) => false,
            MonitorSpecificWallpapers::Specific(wallpapers) => wallpapers.is_empty(),
        }
    }

    pub fn get(&self, monitor_id: &str) -> Option<&String> {
        match self {
            MonitorSpecificWallpapers::All(theme_id) => Some(theme_id),
            MonitorSpecificWallpapers::Specific(wallpapers) => wallpapers.get(monitor_id),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    github_mirror_template: Option<String>,

    #[deprecated(since = "0.1.21", note = "Use `monitor_specific_wallpapers` instead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_theme_id: Option<String>,

    #[serde(default = "default_image_format")]
    image_format: ImageFormat,

    #[serde(default = "default_coordinate_source")]
    coordinate_source: CoordinateSource,

    #[serde(default = "default_auto_detect_color_mode")]
    auto_detect_color_mode: bool,

    #[serde(default = "default_lock_screen_wallpaper_enabled")]
    lock_screen_wallpaper_enabled: bool,

    #[serde(default = "default_themes_directory")]
    themes_directory: PathBuf,

    /// Wallpapers specific to each monitor, using monitor ID as key
    #[serde(default = "default_monitor_specific_wallpapers")]
    monitor_specific_wallpapers: MonitorSpecificWallpapers,

    /// Time interval for detecting solar altitude angle and azimuth angle
    /// Measured in seconds, range: [1, 600]
    #[serde(default = "default_interval")]
    #[validate(minimum = 1)]
    #[validate(maximum = 600)]
    interval: u16,
}

fn default_image_format() -> ImageFormat {
    Default::default()
}

fn default_coordinate_source() -> CoordinateSource {
    Default::default()
}

fn default_auto_detect_color_mode() -> bool {
    true
}

fn default_lock_screen_wallpaper_enabled() -> bool {
    true
}

fn default_interval() -> u16 {
    15
}

fn default_themes_directory() -> PathBuf {
    DWALL_CONFIG_DIR.join("themes")
}

fn default_monitor_specific_wallpapers() -> MonitorSpecificWallpapers {
    MonitorSpecificWallpapers::Specific(HashMap::new())
}

impl Config {
    /// Validates the configuration values
    ///
    /// Checks if the interval is within the valid range and if the coordinate source is valid
    pub fn validate(&self) -> DwallResult<()> {
        if self.interval < 1 || self.interval > 600 {
            error!(interval = self.interval, "Interval validation failed");
            return Err(ConfigError::Validation.into());
        }

        if !self.coordinate_source.validate() {
            error!("Latitude or longitude is invalid");
            return Err(ConfigError::Validation.into());
        }

        Ok(())
    }

    /// Returns the themes directory path
    pub fn themes_directory(&self) -> &Path {
        &self.themes_directory
    }

    /// Creates a new Config with a different themes directory
    pub fn with_themes_directory(&self, themes_directory: &Path) -> Config {
        let mut config = self.clone();
        config.themes_directory = themes_directory.to_path_buf();
        config
    }

    /// Returns the update interval in seconds
    pub fn interval(&self) -> u16 {
        self.interval
    }

    /// Returns the image format
    pub fn image_format(&self) -> &ImageFormat {
        &self.image_format
    }

    /// Returns whether auto detection of color mode is enabled
    pub fn auto_detect_color_mode(&self) -> bool {
        self.auto_detect_color_mode
    }

    /// Returns whether lock screen wallpaper is enabled
    pub fn lock_screen_wallpaper_enabled(&self) -> bool {
        self.lock_screen_wallpaper_enabled
    }

    /// Returns the coordinate source
    pub fn coordinate_source(&self) -> &CoordinateSource {
        &self.coordinate_source
    }

    /// Returns the monitor-specific wallpapers map
    pub fn monitor_specific_wallpapers(&self) -> &MonitorSpecificWallpapers {
        &self.monitor_specific_wallpapers
    }

    /// Converts a GitHub URL to a mirrored URL if a mirror template is configured
    pub fn github_asset_url(&self, github_url: &str) -> String {
        self.github_mirror_template
            .as_ref()
            .and_then(|v| if v.is_empty() { None } else { Some(v) })
            .and_then(|template| {
                use regex::Regex;

                let re = Regex::new(
                    r"https://github.com/([^/]+)/([^/]+)/releases/download/([^/]+)/(.*)",
                )
                .unwrap();
                re.captures(github_url).map(|caps| {
                    template
                        .replace("<owner>", &caps[1])
                        .replace("<repo>", &caps[2])
                        .replace("<version>", &caps[3])
                        .replace("<asset>", &caps[4])
                })
            })
            .unwrap_or(github_url.to_owned())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            image_format: Default::default(),
            coordinate_source: Default::default(),
            github_mirror_template: Default::default(),
            selected_theme_id: Default::default(),
            auto_detect_color_mode: default_auto_detect_color_mode(),
            themes_directory: default_themes_directory(),
            lock_screen_wallpaper_enabled: default_lock_screen_wallpaper_enabled(),
            monitor_specific_wallpapers: default_monitor_specific_wallpapers(),
            // On the equator, an azimuth change of 0.1 degrees takes
            // approximately 12 seconds, and an altitude change of 0.1
            // degrees takes about 24 seconds.
            // Set the default time interval to 15 seconds based on the
            // rate of change of the azimuth.
            // FIXME: This default value is a rough estimate; a more
            // precise algorithm should be used to calculate the time
            // interval required for each 0.1 degree change.
            interval: 15,
        }
    }
}

/// Manages configuration file operations
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new ConfigManager with the given config directory
    pub fn new(config_dir: &Path) -> Self {
        Self {
            config_path: config_dir.join("config.toml"),
        }
    }

    /// Reads the configuration from the file system
    ///
    /// Returns the default configuration if the file doesn't exist
    pub async fn read_config(&self) -> DwallResult<Config> {
        // Return default configuration if config file does not exist
        if !self.config_path.exists() {
            warn!("Config file not found, using default configuration");
            return Ok(Config::default());
        }

        debug!(path = %self.config_path.display(), "Reading configuration file");

        let content = tokio::fs::read_to_string(&self.config_path).await?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            error!(error = ?e, "Failed to parse configuration");
            ConfigError::Deserialization(e)
        })?;

        // Validate configuration
        config.validate()?;

        info!("Local configuration: {:?}", config);

        Ok(config)
    }

    /// Writes the configuration to the file system
    ///
    /// Skips writing if the configuration is unchanged
    pub async fn write_config(&self, config: &Config) -> DwallResult<()> {
        // Validate configuration before writing
        config.validate()?;

        // If file doesn't exist, write directly
        if !self.config_path.exists() {
            return self.write_config_to_file(config).await;
        }

        // Check if config has changed
        if let Ok(existing_config) = self.read_config().await {
            if existing_config == *config {
                debug!("Configuration unchanged, skipping write");
                return Ok(());
            }
        }

        self.write_config_to_file(config).await
    }

    /// Writes the configuration to the file
    async fn write_config_to_file(&self, config: &Config) -> DwallResult<()> {
        let toml_string = toml::to_string(config).map_err(|e| {
            error!(error = ?e, "Failed to serialize configuration");
            ConfigError::Serialization(e)
        })?;

        info!(path = %self.config_path.display(), "Writing configuration file");

        tokio::fs::write(&self.config_path, toml_string.as_bytes()).await?;
        Ok(())
    }
}

/// Reads the configuration file from the default location
pub async fn read_config_file() -> DwallResult<Config> {
    let config_manager = ConfigManager::new(&DWALL_CONFIG_DIR);
    config_manager.read_config().await
}

/// Writes the configuration to the default location
pub async fn write_config_file(config: &Config) -> DwallResult<()> {
    let config_manager = ConfigManager::new(&DWALL_CONFIG_DIR);
    config_manager.write_config(config).await
}
