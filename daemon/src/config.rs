use std::{
    borrow::Cow,
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

    #[error("Config file not found or inaccessible")]
    FileNotFound,
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq)]
pub struct Config<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    github_mirror_template: Option<Cow<'a, str>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    selected_theme_id: Option<Cow<'a, str>>,

    #[serde(default = "default_image_format")]
    image_format: ImageFormat,

    #[serde(default = "default_coordinate_source")]
    coordinate_source: CoordinateSource,

    #[serde(default = "default_auto_detect_color_mode")]
    auto_detect_color_mode: bool,

    #[serde(default = "default_lock_screen_wallpaper_enabled")]
    lock_screen_wallpaper_enabled: bool,

    #[serde(default = "default_themes_directory")]
    themes_directory: Cow<'a, Path>,

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

fn default_themes_directory<'a>() -> Cow<'a, Path> {
    DWALL_CONFIG_DIR.join("themes").into()
}

impl<'a> Config<'a> {
    pub fn validate(&self) -> DwallResult<()> {
        if self.interval < 1 || self.interval > 600 {
            error!(interval = self.interval, "Interval validation failed");
            return Err(ConfigError::Validation.into());
        }
        Ok(())
    }

    pub fn theme_id(&self) -> Option<&str> {
        self.selected_theme_id.as_deref()
    }

    pub fn themes_directory(&self) -> &Path {
        &self.themes_directory
    }

    pub fn with_themes_directory(&self, themes_directory: &'a Path) -> Config<'a> {
        let mut config = self.clone();
        config.themes_directory = themes_directory.into();
        config
    }

    pub fn interval(&self) -> u16 {
        self.interval
    }

    pub fn image_format(&self) -> &ImageFormat {
        &self.image_format
    }

    pub fn auto_detect_color_mode(&self) -> bool {
        self.auto_detect_color_mode
    }

    pub fn lock_screen_wallpaper_enabled(&self) -> bool {
        self.lock_screen_wallpaper_enabled
    }

    pub fn coordinate_source(&'a self) -> &'a CoordinateSource {
        &self.coordinate_source
    }

    pub fn github_asset_url(&self, github_url: &'a str) -> String {
        self.github_mirror_template
            .as_ref()
            .and_then(|v| if v == "" { None } else { Some(v) })
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

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            image_format: Default::default(),
            coordinate_source: Default::default(),
            github_mirror_template: Default::default(),
            selected_theme_id: Default::default(),
            auto_detect_color_mode: default_auto_detect_color_mode(),
            themes_directory: default_themes_directory(),
            lock_screen_wallpaper_enabled: default_lock_screen_wallpaper_enabled(),
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

pub struct ConfigManager {
    config_path: PathBuf,
}

impl<'a> ConfigManager {
    pub fn new(config_dir: &'a Path) -> Self {
        Self {
            config_path: config_dir.join("config.toml"),
        }
    }

    pub async fn read_config(&self) -> DwallResult<Config<'a>> {
        // Return default configuration if config file does not exist
        if !self.config_path.exists() {
            warn!("Config file not found, using default configuration");
            return Ok(Config::default());
        }

        debug!(path = %self.config_path.display(), "Reading configuration file");

        let content = tokio::fs::read_to_string(&self.config_path).await?;

        let config: Config = match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                error!(error = ?e, "Failed to parse configuration");
                return Err(ConfigError::Deserialization(e).into());
            }
        };

        // Validate configuration
        config.validate()?;

        info!("Local configuration: {:?}", config);

        Ok(config)
    }

    pub async fn write_config(&self, config: &Config<'a>) -> DwallResult<()> {
        // Validate configuration before writing
        config.validate()?;

        if !self.config_path.exists() {
            return self.write_config_to_file(config).await;
        }

        let existing_config = self.read_config().await?;

        if existing_config == *config {
            debug!("Configuration unchanged, skipping write");
            return Ok(());
        }

        self.write_config_to_file(config).await
    }

    async fn write_config_to_file(&self, config: &Config<'a>) -> DwallResult<()> {
        let toml_string = match toml::to_string(config) {
            Ok(s) => s,
            Err(e) => {
                error!(error = ?e, "Failed to serialize configuration");
                return Err(ConfigError::Serialization(e).into());
            }
        };

        info!(path = %self.config_path.display(), "Writing configuration file");

        tokio::fs::write(&self.config_path, toml_string.as_bytes()).await?;
        Ok(())
    }
}

pub async fn read_config_file<'a>() -> DwallResult<Config<'a>> {
    let config_manager = ConfigManager::new(&DWALL_CONFIG_DIR);
    config_manager.read_config().await
}

pub async fn write_config_file(config: &Config<'_>) -> DwallResult<()> {
    let config_manager = ConfigManager::new(&DWALL_CONFIG_DIR);
    config_manager.write_config(config).await
}
