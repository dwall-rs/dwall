use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_valid::Validate;

use crate::{
    error::{ConfigError, DwallResult},
    lazy::DWALL_CONFIG_DIR,
};

// Configuration constants
const DEFAULT_INTERVAL_SECONDS: u16 = 15;
const MIN_INTERVAL_SECONDS: u16 = 1;
const MAX_INTERVAL_SECONDS: u16 = 600;
const DEFAULT_AUTO_DETECT_COLOR_MODE: bool = true;
const DEFAULT_LOCK_SCREEN_WALLPAPER_ENABLED: bool = true;

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
    /// Measured in seconds, range: [MIN_INTERVAL_SECONDS, MAX_INTERVAL_SECONDS]
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
    DEFAULT_AUTO_DETECT_COLOR_MODE
}

fn default_lock_screen_wallpaper_enabled() -> bool {
    DEFAULT_LOCK_SCREEN_WALLPAPER_ENABLED
}

fn default_interval() -> u16 {
    DEFAULT_INTERVAL_SECONDS
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
        if self.interval < MIN_INTERVAL_SECONDS || self.interval > MAX_INTERVAL_SECONDS {
            error!(
                interval = self.interval,
                min = MIN_INTERVAL_SECONDS,
                max = MAX_INTERVAL_SECONDS,
                "Interval validation failed"
            );
            return Err(ConfigError::Validation {
                reason: "Interval is out of range".to_string(),
            }
            .into());
        }

        if !self.coordinate_source.validate() {
            error!("Latitude or longitude is invalid");
            return Err(ConfigError::Validation {
                reason: "Latitude or longitude is invalid".to_string(),
            }
            .into());
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

    /// Resolves a GitHub asset URL to a mirrored URL if a mirror template is configured
    ///
    /// This method transforms GitHub release URLs using the configured mirror template,
    /// replacing placeholders like `<owner>`, `<repo>`, `<version>`, and `<asset>`.
    pub fn resolve_github_mirror_url(&self, github_url: &str) -> String {
        self.github_mirror_template
            .as_ref()
            .and_then(|v| if v.is_empty() { None } else { Some(v.as_str()) })
            .and_then(|template| {
                // Parse GitHub URL: https://github.com/{owner}/{repo}/releases/download/{version}/{asset}
                let prefix = "https://github.com/";
                if !github_url.starts_with(prefix) {
                    return None;
                }

                let remaining = &github_url[prefix.len()..];
                let parts: Vec<&str> = remaining.split('/').collect();

                // Expected format: {owner}/{repo}/releases/download/{version}/{asset}
                if parts.len() >= 5 && parts[2] == "releases" && parts[3] == "download" {
                    let owner = parts[0];
                    let repo = parts[1];
                    let version = parts[4];
                    // Asset might contain slashes, so join the remaining parts
                    let asset = parts[5..].join("/");

                    Some(
                        template
                            .replace("<owner>", owner)
                            .replace("<repo>", repo)
                            .replace("<version>", version)
                            .replace("<asset>", &asset),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_else(|| github_url.to_owned())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            image_format: Default::default(),
            coordinate_source: Default::default(),
            github_mirror_template: Default::default(),
            auto_detect_color_mode: default_auto_detect_color_mode(),
            themes_directory: default_themes_directory(),
            lock_screen_wallpaper_enabled: default_lock_screen_wallpaper_enabled(),
            monitor_specific_wallpapers: default_monitor_specific_wallpapers(),
            // On the equator, an azimuth change of 0.1 degrees takes
            // approximately 12 seconds, and an altitude change of 0.1
            // degrees takes about 24 seconds.
            // Set the default time interval based on the rate of change of the azimuth.
            // On the equator, an azimuth change of 0.1 degrees takes approximately 12 seconds,
            // and an altitude change of 0.1 degrees takes about 24 seconds.
            // FIXME: This default value is a rough estimate; a more precise algorithm should
            // be used to calculate the time interval required for each 0.1 degree change.
            interval: DEFAULT_INTERVAL_SECONDS,
        }
    }
}
