use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{ConfigError, DwallResult},
    lazy::DWALL_CONFIG_DIR,
};

// Configuration constants
const DEFAULT_INTERVAL_SECONDS: u16 = 15;
const MIN_INTERVAL_SECONDS: u16 = 1;
const MAX_INTERVAL_SECONDS: u16 = 3600;
const DEFAULT_AUTO_DETECT_COLOR_SCHEME: bool = true;
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
pub enum PositionSource {
    Automatic {
        #[serde(default = "default_update_on_each_calculation")]
        update_on_each_calculation: bool,
        #[serde(default = "default_position_cache_minutes")]
        cache_minutes: u64,
    },

    Manual {
        latitude: f64,
        longitude: f64,
        #[serde(default = "default_altitude")]
        altitude: f64,
    },
}

fn default_altitude() -> f64 {
    // Some sources cite the average elevation of land as 840 meters;
    // however, among mainstream authoritative geographic or Earth
    // science institutions (such as National Geographic, NASA, NOAA,
    // or relevant United Nations reports), the figure of 875 meters
    // is more widely accepted and commonly cited. Therefore,
    // 875 meters is used here as the default value.
    875.
}

impl Default for PositionSource {
    fn default() -> Self {
        Self::Automatic {
            update_on_each_calculation: false,
            cache_minutes: default_position_cache_minutes(),
        }
    }
}

fn default_update_on_each_calculation() -> bool {
    false
}

impl PositionSource {
    pub fn validate(&self) -> bool {
        match *self {
            PositionSource::Automatic { .. } => true,
            PositionSource::Manual {
                latitude,
                longitude,
                ..
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum Network {
    GitHubMirrorTemplate(String),
    Socks5 { host: String, port: u16 },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<Network>,

    #[serde(default = "default_image_format")]
    image_format: ImageFormat,

    #[serde(alias = "coordinate_source", default = "default_position_source")]
    position_source: PositionSource,

    #[serde(
        alias = "auto_detect_color_mode",
        default = "default_auto_detect_color_scheme"
    )]
    auto_detect_color_scheme: bool,

    #[serde(default = "default_lock_screen_wallpaper_enabled")]
    lock_screen_wallpaper_enabled: bool,

    #[serde(default = "default_themes_directory")]
    themes_directory: PathBuf,

    #[serde(default = "default_customized_themes_directory")]
    customized_themes_directory: PathBuf,

    /// Wallpapers specific to each monitor, using monitor ID as key
    #[serde(default = "default_monitor_specific_wallpapers")]
    monitor_specific_wallpapers: MonitorSpecificWallpapers,

    /// Time interval for detecting solar altitude angle and azimuth angle
    /// Measured in seconds, range: `[MIN_INTERVAL_SECONDS, MAX_INTERVAL_SECONDS]`
    #[serde(
        default = "default_interval",
        deserialize_with = "deserialize_interval"
    )]
    interval: u16,
}

fn default_image_format() -> ImageFormat {
    Default::default()
}

fn default_position_source() -> PositionSource {
    Default::default()
}

fn default_position_cache_minutes() -> u64 {
    30
}

fn default_auto_detect_color_scheme() -> bool {
    DEFAULT_AUTO_DETECT_COLOR_SCHEME
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

fn default_customized_themes_directory() -> PathBuf {
    DWALL_CONFIG_DIR.join("customize")
}

fn default_monitor_specific_wallpapers() -> MonitorSpecificWallpapers {
    MonitorSpecificWallpapers::Specific(HashMap::new())
}

/// Custom deserializer for interval field with range validation
fn deserialize_interval<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = u16::deserialize(deserializer)?;
    if !(MIN_INTERVAL_SECONDS..=MAX_INTERVAL_SECONDS).contains(&value) {
        return Err(serde::de::Error::custom(format!(
            "interval must be between {} and {}, got {}",
            MIN_INTERVAL_SECONDS, MAX_INTERVAL_SECONDS, value
        )));
    }
    Ok(value)
}

impl Config {
    /// Validates the configuration values
    ///
    /// Checks if the interval is within the valid range and if the coordinate source is valid
    pub fn validate(&self) -> DwallResult<()> {
        if self.interval < MIN_INTERVAL_SECONDS || self.interval > MAX_INTERVAL_SECONDS {
            error!(
                "Interval validation failed: min={MIN_INTERVAL_SECONDS}, max={MAX_INTERVAL_SECONDS}, interval={}",
                self.interval
            );
            return Err(ConfigError::Validation {
                reason: "Interval is out of range".to_string(),
            }
            .into());
        }

        if !self.position_source.validate() {
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

    /// Returns the customized themes directory path
    pub fn customized_themes_directory(&self) -> &Path {
        &self.customized_themes_directory
    }

    /// Returns the network configuration
    pub fn network(&self) -> Option<&Network> {
        self.network.as_ref()
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
    pub fn auto_detect_color_scheme(&self) -> bool {
        self.auto_detect_color_scheme
    }

    /// Returns whether lock screen wallpaper is enabled
    pub fn lock_screen_wallpaper_enabled(&self) -> bool {
        self.lock_screen_wallpaper_enabled
    }

    /// Returns the position source
    pub fn position_source(&self) -> &PositionSource {
        &self.position_source
    }

    /// Returns the monitor-specific wallpapers map
    pub fn monitor_specific_wallpapers(&self) -> &MonitorSpecificWallpapers {
        &self.monitor_specific_wallpapers
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            image_format: Default::default(),
            network: Default::default(),
            position_source: Default::default(),
            auto_detect_color_scheme: default_auto_detect_color_scheme(),
            themes_directory: default_themes_directory(),
            customized_themes_directory: default_customized_themes_directory(),
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

/// Intermediate deserialization struct for backward compatibility with legacy configuration files.
///
/// The `github_mirror_template` field was deprecated in 0.2.0 and will be removed in 0.3.0.
/// At that time, this struct should also be removed, and the standard `Deserialize` derivation of `Config` should be restored.
#[derive(Debug, Deserialize)]
pub struct RawConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    github_mirror_template: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<Network>,

    #[serde(default = "default_image_format")]
    image_format: ImageFormat,

    #[serde(alias = "coordinate_source", default = "default_position_source")]
    position_source: PositionSource,

    #[serde(
        alias = "auto_detect_color_mode",
        default = "default_auto_detect_color_scheme"
    )]
    auto_detect_color_scheme: bool,

    #[serde(default = "default_lock_screen_wallpaper_enabled")]
    lock_screen_wallpaper_enabled: bool,

    #[serde(default = "default_themes_directory")]
    themes_directory: PathBuf,

    #[serde(default = "default_customized_themes_directory")]
    customized_themes_directory: PathBuf,

    /// Wallpapers specific to each monitor, using monitor ID as key
    #[serde(default = "default_monitor_specific_wallpapers")]
    monitor_specific_wallpapers: MonitorSpecificWallpapers,

    /// Time interval for detecting solar altitude angle and azimuth angle
    /// Measured in seconds, range: `[MIN_INTERVAL_SECONDS, MAX_INTERVAL_SECONDS]`
    #[serde(
        default = "default_interval",
        deserialize_with = "deserialize_interval"
    )]
    interval: u16,
}

impl From<RawConfig> for Config {
    fn from(raw: RawConfig) -> Self {
        // Migrate github_mirror_template
        let network = raw.network.or_else(|| {
            raw.github_mirror_template
                .map(Network::GitHubMirrorTemplate)
        });

        Config {
            network,
            image_format: raw.image_format,
            position_source: raw.position_source,
            auto_detect_color_scheme: raw.auto_detect_color_scheme,
            lock_screen_wallpaper_enabled: raw.lock_screen_wallpaper_enabled,
            themes_directory: raw.themes_directory,
            customized_themes_directory: raw.customized_themes_directory,
            monitor_specific_wallpapers: raw.monitor_specific_wallpapers,
            interval: raw.interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialize() {
        let config_str = r#"
        {
            "interval": 60
        }
        "#;
        let result = serde_json::from_str(config_str);
        assert!(result.is_ok());
        let config: Config = result.unwrap();
        assert_eq!(config.interval, 60);

        let config_str = r#"
        {
            "interval": 3601
        }
        "#;
        let result = serde_json::from_str::<Config>(config_str);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.starts_with("interval must be between 1 and 3600, got 3601"));
    }
}
