use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{ConfigError, DwallResult},
    lazy::DWALL_CONFIG_DIR,
};

// Configuration constants

/// How often the daemon recomputes the sun's position and re-checks the wallpaper.
///
/// Derived, not guessed: over a year at latitudes 0–65° the sun's altitude moves at
/// most 0.0233°/s (at the equator in March, where it also crosses the zenith), so 15
/// seconds lets the applied frame lag the sun by at most 0.35° of altitude. Expressed
/// in the selector's own metric (`wallpaper::solar_distance`, altitude weighted twice
/// the azimuth) that is 0.0053, which is 10% of the closest frame pair measured in
/// real themes (0.0529, a 24-frame theme) — the switch happens before the sun has
/// moved a sixth of the way from one frame to its neighbour. 30 seconds would reach
/// 20%, which is why doubling this value is not free.
/// `the_default_interval_keeps_the_frame_within_a_tenth_of_its_neighbour` pins it.
const DEFAULT_INTERVAL_SECONDS: u16 = 15;
const MIN_INTERVAL_SECONDS: u16 = 1;
const MAX_INTERVAL_SECONDS: u16 = 3600;

/// How long an automatically obtained position may be reused.
///
/// The daemon cannot tell whether the machine moved, so this trades the accuracy of a
/// laptop that travels against how often Windows has to read the location sensor
/// (a refresh forces a real reading: the platform's own cache only covers 60 seconds,
/// `geolocation::MAX_GEOPOSITION_AGE`). Measured over a day, a 30-minute-old reading
/// after 50 km of travel — 100 km/h — picks a neighbouring frame 0.5–2.9% of the time,
/// which is the dominant case; an aircraft covering 450 km in the same window reaches
/// 4–30%, and there the frames are visibly wrong until the next refresh. Refresh cost
/// is not the constraint: 245 forced readings in five minutes added 0.2 MB and ten
/// handles, and each took ~0.2 s.
const DEFAULT_POSITION_CACHE_MINUTES: u64 = 30;

const DEFAULT_AUTO_DETECT_COLOR_SCHEME: bool = true;
const DEFAULT_LOCK_SCREEN_WALLPAPER_ENABLED: bool = true;
const DEFAULT_TITLE_BAR_COLOR_FOLLOWS_WINDOWS_THEME: bool = false;

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    #[default]
    Jpeg,
    Png,
}

impl ImageFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
        }
    }
}

// impl From<&ImageFormat> for &str {
//     fn from(val: &ImageFormat) -> Self {
//         match val {
//             ImageFormat::Jpeg => "jpg",
//             ImageFormat::Png => "png",
//         }
//     }
// }

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
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
            PositionSource::Automatic { cache_minutes, .. } => {
                position_cache_minutes_are_valid(cache_minutes)
            }
            PositionSource::Manual {
                latitude,
                longitude,
                ..
            } => (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude),
        }
    }
}

/// Upper bound for `cache_minutes`.
///
/// The cache TTL is `60 * cache_minutes` seconds, added to an `Instant`. Nothing
/// bounded the value, so a hand-edited `config.toml` with a huge number overflowed
/// that addition and aborted the daemon — in a release build with `panic = "abort"`
/// and a GUI subsystem, leaving no trace at all.
const MAX_POSITION_CACHE_MINUTES: u64 = 24 * 60;

/// Whether a `cache_minutes` value is usable.
fn position_cache_minutes_are_valid(minutes: u64) -> bool {
    minutes <= MAX_POSITION_CACHE_MINUTES
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum MonitorSpecificWallpapers {
    All(String),
    #[serde(alias = "specific")]
    Individual(HashMap<String, String>),
}

impl MonitorSpecificWallpapers {
    pub fn is_all(&self) -> bool {
        matches!(self, MonitorSpecificWallpapers::All(_))
    }

    pub fn is_empty(&self) -> bool {
        match self {
            MonitorSpecificWallpapers::All(_) => false,
            MonitorSpecificWallpapers::Individual(wallpapers) => wallpapers.is_empty(),
        }
    }

    pub fn get(&self, monitor_id: &str) -> Option<&String> {
        match self {
            MonitorSpecificWallpapers::All(theme_id) => Some(theme_id),
            MonitorSpecificWallpapers::Individual(wallpapers) => wallpapers.get(monitor_id),
        }
    }
}

/// Wallpaper switching mode.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case", tag = "mode")]
pub enum WallpaperMode {
    /// Fixed theme mode: the user configures a theme per monitor, and wallpapers switch by solar position.
    Fixed {
        #[serde(default = "default_monitor_specific_wallpapers")]
        monitor_specific_wallpapers: MonitorSpecificWallpapers,
    },
    /// Random theme mode: a theme is picked at random each day, shared by all monitors.
    Random {
        /// Random pool (`None` means use all available themes).
        #[serde(default)]
        pool: Option<Vec<String>>,
    },
}

impl Default for WallpaperMode {
    fn default() -> Self {
        WallpaperMode::Fixed {
            monitor_specific_wallpapers: MonitorSpecificWallpapers::Individual(HashMap::new()),
        }
    }
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum Network {
    GitHubMirrorTemplate(String),
    Socks5 { host: String, port: u16 },
}

#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    /// Network configuration for specifying a GitHub mirror template or SOCKS5 proxy.
    ///
    /// Only used in dwall-settings.
    #[serde(skip_serializing_if = "Option::is_none")]
    network: Option<Network>,

    /// Title bar follows the Windows theme color.
    ///
    /// By default, the program sets the same color as the window background. Users can manually disable this to use the Windows theme color.
    ///
    /// Only used in dwall-settings.
    #[serde(default = "default_title_bar_color_follows_windows_theme")]
    title_bar_color_follows_windows_theme: bool,

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

    /// Wallpaper mode (Fixed or Random)
    #[serde(default)]
    wallpaper_mode: WallpaperMode,

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

fn default_title_bar_color_follows_windows_theme() -> bool {
    DEFAULT_TITLE_BAR_COLOR_FOLLOWS_WINDOWS_THEME
}

fn default_position_source() -> PositionSource {
    Default::default()
}

fn default_position_cache_minutes() -> u64 {
    DEFAULT_POSITION_CACHE_MINUTES
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
    MonitorSpecificWallpapers::Individual(HashMap::new())
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
            // The two cases need different messages: "latitude or longitude" would
            // send someone editing a manual position after a cache setting.
            let reason = match &self.position_source {
                PositionSource::Automatic { cache_minutes, .. }
                    if !position_cache_minutes_are_valid(*cache_minutes) =>
                {
                    error!(
                        cache_minutes = cache_minutes,
                        max = MAX_POSITION_CACHE_MINUTES,
                        "Position cache validation failed"
                    );
                    "Position cache is out of range".to_string()
                }
                _ => {
                    error!("Latitude or longitude is invalid");
                    "Latitude or longitude is invalid".to_string()
                }
            };

            return Err(ConfigError::Validation { reason }.into());
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

    /// Returns the wallpaper mode
    pub fn wallpaper_mode(&self) -> &WallpaperMode {
        &self.wallpaper_mode
    }

    /// Returns the monitor-specific wallpapers map (only for Fixed mode)
    pub fn monitor_specific_wallpapers(&self) -> Option<&MonitorSpecificWallpapers> {
        match &self.wallpaper_mode {
            WallpaperMode::Fixed {
                monitor_specific_wallpapers,
            } => Some(monitor_specific_wallpapers),
            WallpaperMode::Random { .. } => None,
        }
    }

    /// Set the wallpaper mode
    pub fn set_wallpaper_mode(&mut self, mode: WallpaperMode) {
        self.wallpaper_mode = mode;
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            network: Default::default(),
            title_bar_color_follows_windows_theme: default_title_bar_color_follows_windows_theme(),
            image_format: Default::default(),
            position_source: Default::default(),
            auto_detect_color_scheme: default_auto_detect_color_scheme(),
            themes_directory: default_themes_directory(),
            customized_themes_directory: default_customized_themes_directory(),
            lock_screen_wallpaper_enabled: default_lock_screen_wallpaper_enabled(),
            wallpaper_mode: Default::default(),
            // The rationale for the interval lives on `DEFAULT_INTERVAL_SECONDS`: the
            // previous comment here claimed 0.1° of azimuth takes 12 seconds at the
            // equator and 0.1° of altitude 24, which the crate's own solar code
            // contradicts (4.3 s and 0.8 s respectively, the latter at local noon when
            // the sun passes the zenith), and it was duplicated twice with a FIXME.
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

    #[serde(default = "default_title_bar_color_follows_windows_theme")]
    title_bar_color_follows_windows_theme: bool,

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

    /// Wallpaper mode (new field for migration)
    #[serde(default)]
    wallpaper_mode: Option<WallpaperMode>,

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

        // Migrate wallpaper_mode: if absent, wrap monitor_specific_wallpapers in Fixed mode
        let wallpaper_mode = raw.wallpaper_mode.unwrap_or(WallpaperMode::Fixed {
            monitor_specific_wallpapers: raw.monitor_specific_wallpapers,
        });

        Config {
            title_bar_color_follows_windows_theme: raw.title_bar_color_follows_windows_theme,
            network,
            image_format: raw.image_format,
            position_source: raw.position_source,
            auto_detect_color_scheme: raw.auto_detect_color_scheme,
            lock_screen_wallpaper_enabled: raw.lock_screen_wallpaper_enabled,
            themes_directory: raw.themes_directory,
            customized_themes_directory: raw.customized_themes_directory,
            wallpaper_mode,
            interval: raw.interval,
        }
    }
}

// ── Config file I/O ─────────────────────────────────────────────────────────

/// Reads configuration from the filesystem.
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

        // Deliberately not the whole struct, at any level: it carries the user's
        // exact coordinates, the network/proxy settings and the profile directory
        // names, and the log directory is one click away in the settings window.
        // (A `debug!(config = ?config)` used to sit here and undid that.)
        info!(
            interval_seconds = config.interval(),
            image_format = ?config.image_format(),
            auto_detect_color_scheme = config.auto_detect_color_scheme(),
            lock_screen_wallpaper_enabled = config.lock_screen_wallpaper_enabled(),
            random_wallpaper_mode = matches!(config.wallpaper_mode(), WallpaperMode::Random { .. }),
            manual_position = matches!(config.position_source(), PositionSource::Manual { .. }),
            proxy_configured = config.network().is_some(),
            "Configuration loaded successfully"
        );

        Ok(config)
    }
}

/// Writes configuration to the filesystem.
pub struct ConfigWriter;

impl ConfigWriter {
    /// Writes configuration to the specified path
    pub fn write_to_path(&self, config_path: &Path, config: &Config) -> DwallResult<()> {
        config.validate()?;

        let toml_string = toml::to_string(config).map_err(|e| {
            error!(error = %e, "Failed to serialize configuration");
            ConfigError::Serialization(e)
        })?;

        info!(path = %config_path.display(), "Writing configuration file");
        write_atomically(config_path, toml_string.as_bytes())?;
        Ok(())
    }

    /// Writes a commented default configuration, creating the parent directory.
    ///
    /// Used when the daemon starts without a configuration file: `ConfigReader`
    /// would otherwise silently hand out an in-memory default, which leaves anyone
    /// running the daemon without the settings window with no file to edit and no
    /// documentation of the keys.
    pub fn write_default(&self, config_path: &Path) -> DwallResult<()> {
        let body = toml::to_string(&Config::default()).map_err(|e| {
            error!(error = %e, "Failed to serialize the default configuration");
            ConfigError::Serialization(e)
        })?;

        let contents = format!(
            "# Dwall configuration\n\
             #\n\
             # Created automatically because no configuration file existed. Every key is\n\
             # optional; a key that is missing falls back to the value shown here.\n\
             #\n\
             #   dwall.exe --check-config     validate an edit without applying anything\n\
             #   dwall.exe --once             apply one update cycle and exit\n\
             #\n\
             # Logs: %LOCALAPPDATA%\\com.thep0y.dwall\\log\\dwall.log\n\
             #       (set DWALL_LOG=debug for more detail)\n\
             #\n\
             # See docs/headless.md for the standalone workflow.\n\
             \n{body}"
        );

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        write_atomically(config_path, contents.as_bytes())?;
        Ok(())
    }
}

/// The default configuration file path (`%APPDATA%\dwall\config.toml`).
pub fn config_path() -> PathBuf {
    DWALL_CONFIG_DIR.join("config.toml")
}

/// Writes `contents` to `path` through a temporary file plus rename.
///
/// The daemon polls this file (and the settings window writes it while the daemon
/// runs), so a plain `fs::write` — truncate, then write — exposes a window in which
/// a reader sees a half-written document. `fs::rename` replaces the destination
/// atomically (MOVEFILE_REPLACE_EXISTING on Windows).
/// The sibling temporary file [`write_atomically`] writes through.
///
/// The configured file's own name is kept, so `--config foo.conf` does not turn
/// into `foo.toml.tmp`, and the process id is appended so two instances pointed
/// at the same directory cannot overwrite each other's temporary file.
fn temporary_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "config.toml".to_string());

    path.with_file_name(format!("{name}.{}.tmp", std::process::id()))
}

fn write_atomically(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let temp_path = temporary_path(path);

    fs::write(&temp_path, contents)?;

    if let Err(e) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    Ok(())
}

/// Watches a configuration file for modification-time changes.
pub(crate) struct ConfigWatcher {
    config_path: PathBuf,
    last_modified: Option<SystemTime>,
}

impl ConfigWatcher {
    /// Creates a new ConfigWatcher for the specified config path
    pub(crate) fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            last_modified: None,
        }
    }

    /// Returns the config path being watched
    pub(crate) fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_file_modified_time(&self) -> DwallResult<Option<SystemTime>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(&self.config_path)?;
        Ok(Some(metadata.modified()?))
    }

    /// Checks if the configuration file has changed since the last check.
    ///
    /// Returns `true` when the file appeared, disappeared, or its modification
    /// time differs from the last known one.
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

    /// Updates the last known modification time
    pub(crate) fn update_modified_time(&mut self) -> DwallResult<()> {
        self.last_modified = self.get_file_modified_time()?;
        Ok(())
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

    // ── Integration tests for serialization/deserialization ────────────────────

    #[test]
    fn write_atomically_replaces_existing_file() {
        let dir = std::env::temp_dir().join("dwall_config_atomic");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        fs::write(&path, "interval = 30").unwrap();
        write_atomically(&path, b"interval = 45").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "interval = 45");
        assert_eq!(
            temporary_path(&path),
            dir.join(format!("config.toml.{}.tmp", std::process::id())),
            "the temporary file keeps the configured name and names the process"
        );
        assert!(
            !temporary_path(&path).exists(),
            "the temporary file is renamed away, not left behind"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// `--config foo.conf` used to be written through `foo.toml.tmp`, so
    /// two instances sharing a directory overwrote each other's temporary file.
    #[test]
    fn the_temporary_file_keeps_the_configured_name_and_names_the_process() {
        let path = Path::new("custom.conf");

        assert_eq!(
            temporary_path(path),
            PathBuf::from(format!("custom.conf.{}.tmp", std::process::id()))
        );
    }

    /// Test full config roundtrip serialization
    #[test]
    fn config_roundtrip_serialization() {
        let original = Config {
            network: Some(Network::Socks5 {
                host: "127.0.0.1".to_string(),
                port: 1080,
            }),
            title_bar_color_follows_windows_theme: true,
            image_format: ImageFormat::Png,
            position_source: PositionSource::Manual {
                latitude: 45.0,
                longitude: 116.0,
                altitude: 100.0,
            },
            auto_detect_color_scheme: false,
            lock_screen_wallpaper_enabled: false,
            themes_directory: PathBuf::from("/tmp/themes"),
            customized_themes_directory: PathBuf::from("/tmp/customize"),
            wallpaper_mode: WallpaperMode::Fixed {
                monitor_specific_wallpapers: MonitorSpecificWallpapers::All("theme1".to_string()),
            },
            interval: 30,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.interval, original.interval);
        assert_eq!(deserialized.image_format, original.image_format);
        assert_eq!(deserialized.network, original.network);
        assert_eq!(deserialized.wallpaper_mode, original.wallpaper_mode);
    }

    /// Test RawConfig migration from legacy github_mirror_template
    #[test]
    fn raw_config_migration_github_mirror() {
        let raw = RawConfig {
            github_mirror_template: Some("https://ghproxy.cc".to_string()),
            network: None,
            title_bar_color_follows_windows_theme: false,
            image_format: ImageFormat::Jpeg,
            position_source: PositionSource::default(),
            auto_detect_color_scheme: true,
            lock_screen_wallpaper_enabled: true,
            themes_directory: PathBuf::from("/tmp/themes"),
            customized_themes_directory: PathBuf::from("/tmp/customize"),
            monitor_specific_wallpapers: MonitorSpecificWallpapers::Individual(
                std::collections::HashMap::new(),
            ),
            wallpaper_mode: None,
            interval: 15,
        };

        let config: Config = raw.into();
        assert_eq!(
            config.network,
            Some(Network::GitHubMirrorTemplate(
                "https://ghproxy.cc".to_string()
            ))
        );
    }

    /// Test RawConfig migration prefers network over github_mirror_template
    #[test]
    fn raw_config_migration_prefers_network() {
        let raw = RawConfig {
            github_mirror_template: Some("https://old.example.com".to_string()),
            network: Some(Network::Socks5 {
                host: "127.0.0.1".to_string(),
                port: 1080,
            }),
            title_bar_color_follows_windows_theme: false,
            image_format: ImageFormat::Jpeg,
            position_source: PositionSource::default(),
            auto_detect_color_scheme: true,
            lock_screen_wallpaper_enabled: true,
            themes_directory: PathBuf::from("/tmp/themes"),
            customized_themes_directory: PathBuf::from("/tmp/customize"),
            monitor_specific_wallpapers: MonitorSpecificWallpapers::Individual(
                std::collections::HashMap::new(),
            ),
            wallpaper_mode: None,
            interval: 15,
        };

        let config: Config = raw.into();
        // network should take precedence
        assert_eq!(
            config.network,
            Some(Network::Socks5 {
                host: "127.0.0.1".to_string(),
                port: 1080,
            })
        );
    }

    /// Test RawConfig migration wraps monitor_specific_wallpapers in Fixed mode
    #[test]
    fn raw_config_migration_wallpaper_mode() {
        let mut map = std::collections::HashMap::new();
        map.insert("monitor1".to_string(), "theme1".to_string());

        let raw = RawConfig {
            github_mirror_template: None,
            network: None,
            title_bar_color_follows_windows_theme: false,
            image_format: ImageFormat::Jpeg,
            position_source: PositionSource::default(),
            auto_detect_color_scheme: true,
            lock_screen_wallpaper_enabled: true,
            themes_directory: PathBuf::from("/tmp/themes"),
            customized_themes_directory: PathBuf::from("/tmp/customize"),
            monitor_specific_wallpapers: MonitorSpecificWallpapers::Individual(map.clone()),
            wallpaper_mode: None, // Old config without wallpaper_mode
            interval: 15,
        };

        let config: Config = raw.into();
        // Should be migrated to Fixed mode
        assert_eq!(
            config.wallpaper_mode,
            WallpaperMode::Fixed {
                monitor_specific_wallpapers: MonitorSpecificWallpapers::Individual(map)
            }
        );
    }

    /// Test MonitorSpecificWallpapers serialization variants
    #[test]
    fn monitor_wallpapers_all_variant() {
        let wallpapers = MonitorSpecificWallpapers::All("theme1".to_string());
        let json = serde_json::to_string(&wallpapers).unwrap();
        assert_eq!(json, r#""theme1""#);

        let deserialized: MonitorSpecificWallpapers = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, wallpapers);
    }

    #[test]
    fn monitor_wallpapers_specific_variant() {
        let mut map = std::collections::HashMap::new();
        map.insert("monitor1".to_string(), "theme1".to_string());
        map.insert("monitor2".to_string(), "theme2".to_string());
        let wallpapers = MonitorSpecificWallpapers::Individual(map);

        let json = serde_json::to_string(&wallpapers).unwrap();
        let deserialized: MonitorSpecificWallpapers = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, wallpapers);
    }

    /// Test PositionSource serialization
    #[test]
    fn position_source_automatic_variant() {
        let source = PositionSource::Automatic {
            update_on_each_calculation: true,
            cache_minutes: 60,
        };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"type\":\"AUTOMATIC\""));

        let deserialized: PositionSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, source);
    }

    #[test]
    fn position_source_manual_variant() {
        let source = PositionSource::Manual {
            latitude: 45.0,
            longitude: 116.0,
            altitude: 100.0,
        };
        let json = serde_json::to_string(&source).unwrap();
        assert!(json.contains("\"type\":\"MANUAL\""));

        let deserialized: PositionSource = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, source);
    }

    /// Test interval boundary validation
    #[test]
    fn interval_boundary_min() {
        let json = r#"{"interval": 1}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.interval, 1);
    }

    #[test]
    fn interval_boundary_max() {
        let json = r#"{"interval": 3600}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.interval, 3600);
    }

    #[test]
    fn interval_out_of_range_zero() {
        let json = r#"{"interval": 0}"#;
        let result = serde_json::from_str::<Config>(json);
        assert!(result.is_err());
    }

    /// Test backward compatibility alias for coordinate_source
    #[test]
    fn coordinate_source_alias() {
        let json = r#"{"coordinate_source": {"type": "AUTOMATIC", "update_on_each_calculation": false, "cache_minutes": 30}}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(matches!(
            config.position_source,
            PositionSource::Automatic { .. }
        ));
    }

    /// Test backward compatibility alias for auto_detect_color_mode
    #[test]
    fn auto_detect_color_mode_alias() {
        let json = r#"{"auto_detect_color_mode": false}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.auto_detect_color_scheme);
    }

    /// Test WallpaperMode::Fixed serialization
    #[test]
    fn wallpaper_mode_fixed_serialization() {
        let mode = WallpaperMode::Fixed {
            monitor_specific_wallpapers: MonitorSpecificWallpapers::All("theme1".to_string()),
        };
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains(r#""mode":"fixed""#));
        assert!(json.contains("theme1"));
    }

    /// Test WallpaperMode::Random serialization
    #[test]
    fn wallpaper_mode_random_serialization() {
        let mode = WallpaperMode::Random {
            pool: Some(vec!["theme1".to_string(), "theme2".to_string()]),
        };
        let json = serde_json::to_string(&mode).unwrap();
        assert!(json.contains(r#""mode":"random""#));
        assert!(json.contains("theme1"));
    }

    /// Test WallpaperMode::Random with null pool
    #[test]
    fn wallpaper_mode_random_null_pool() {
        let json = r#"{"mode":"random","pool":null}"#;
        let mode: WallpaperMode = serde_json::from_str(json).unwrap();
        assert!(matches!(mode, WallpaperMode::Random { pool: None }));
    }

    /// Test WallpaperMode default is Fixed
    #[test]
    fn wallpaper_mode_default_is_fixed() {
        let mode = WallpaperMode::default();
        assert!(matches!(mode, WallpaperMode::Fixed { .. }));
    }

    /// Test old config without wallpaper_mode migrates to Fixed
    #[test]
    fn old_config_migrates_to_fixed_mode() {
        // Old config format: no wallpaper_mode field
        let json = r#"{"monitor_specific_wallpapers": "theme1"}"#;
        let raw: RawConfig = serde_json::from_str(json).unwrap();
        let config: Config = raw.into();

        assert!(matches!(
            config.wallpaper_mode(),
            WallpaperMode::Fixed { .. }
        ));
    }

    /// Test new config with wallpaper_mode is preserved
    #[test]
    fn new_config_preserves_wallpaper_mode() {
        let json = r#"{
            "wallpaper_mode": {
                "mode": "random",
                "pool": ["theme1", "theme2"]
            }
        }"#;
        let raw: RawConfig = serde_json::from_str(json).unwrap();
        let config: Config = raw.into();

        match config.wallpaper_mode() {
            WallpaperMode::Random { pool } => {
                assert_eq!(pool.as_ref().unwrap().len(), 2);
            }
            _ => panic!("Expected Random mode"),
        }
    }

    /// Test monitor_specific_wallpapers() returns None for Random mode
    #[test]
    fn monitor_specific_wallpapers_none_for_random_mode() {
        let config = Config {
            wallpaper_mode: WallpaperMode::Random { pool: None },
            ..Config::default()
        };
        assert!(config.monitor_specific_wallpapers().is_none());
    }

    /// Test monitor_specific_wallpapers() returns Some for Fixed mode
    #[test]
    fn monitor_specific_wallpapers_some_for_fixed_mode() {
        let config = Config {
            wallpaper_mode: WallpaperMode::Fixed {
                monitor_specific_wallpapers: MonitorSpecificWallpapers::All("theme1".to_string()),
            },
            ..Config::default()
        };
        assert!(config.monitor_specific_wallpapers().is_some());
    }

    // ── Snapshot tests ────────────────────────────────────────────────────────

    #[test]
    fn snapshot_default_config() {
        let config = Config::default();
        insta::assert_json_snapshot!("default_config", config, {
            ".themes_directory" => "[themes_directory]",
            ".customized_themes_directory" => "[customized_themes_directory]",
        });
    }

    // ── cache_minutes validation ──────────────────────────────────────────────

    fn config_with_cache_minutes(cache_minutes: u64) -> Config {
        let json = format!(
            r#"{{"position_source":{{"type":"AUTOMATIC","update_on_each_calculation":false,"cache_minutes":{cache_minutes}}}}}"#
        );

        serde_json::from_str(&json).unwrap()
    }

    /// The regression test for a value that used to abort the daemon: an
    /// out-of-range `cache_minutes` overflowed the TTL arithmetic, and a panic in
    /// this process is invisible.
    #[test]
    fn validate_rejects_an_out_of_range_cache_minutes() {
        for cache_minutes in [MAX_POSITION_CACHE_MINUTES + 1, u64::MAX] {
            let error = config_with_cache_minutes(cache_minutes)
                .validate()
                .expect_err("a cache that never expires is not a cache");

            assert!(
                error.to_string().contains("Position cache"),
                "the message has to name the setting: {error}"
            );
        }
    }

    #[test]
    fn validate_accepts_the_documented_cache_minutes() {
        for cache_minutes in [0, 1, 30, MAX_POSITION_CACHE_MINUTES] {
            assert!(
                config_with_cache_minutes(cache_minutes).validate().is_ok(),
                "cache_minutes = {cache_minutes} should be usable"
            );
        }
    }

    /// The tightest frame spacing to design against, in the selector's metric
    /// (`wallpaper::solar_distance`): the closest pair measured in real themes was
    /// 0.0529 (a 24-frame theme), so frames can be no closer than this.
    const TIGHTEST_FRAME_SPACING: f64 = 0.05;

    /// The worst-case angular rate of the sun over a year at latitudes 0–65°,
    /// in degrees per second, for altitude and azimuth.
    ///
    /// This is the quantity the detection interval has to be compared against, and it
    /// is deliberately recomputed here rather than quoted: the comment that used to
    /// justify the default was wrong by an order of magnitude (it claimed 0.1° of
    /// azimuth takes 12 seconds; at the equator in March the sun crosses the zenith
    /// and sweeps 0.1° of azimuth in 0.8 seconds).
    fn worst_case_solar_rates() -> (f64, f64) {
        let mut worst_altitude = 0.0f64;
        let mut worst_azimuth = 0.0f64;

        for latitude in [0.0f64, 15.0, 30.0, 45.0, 55.0, 60.0, 65.0] {
            let position = crate::solar::Position::new(latitude, 0.0, 0.0).unwrap();

            for month in [
                time::Month::January,
                time::Month::March,
                time::Month::April,
                time::Month::June,
                time::Month::September,
                time::Month::December,
            ] {
                let mut previous: Option<(f64, f64)> = None;

                // A minute of resolution is enough: the rate is smooth, and the peak
                // is broad enough that a finer step moves it by well under a percent.
                for second in (0..86_400u32).step_by(60) {
                    let date = time::UtcDateTime::new(
                        2026,
                        month,
                        15,
                        (second / 3600) as u8,
                        ((second % 3600) / 60) as u8,
                        0,
                    )
                    .unwrap();
                    let solar = crate::solar::SolarPosition::new(&position, &date);
                    let current = (solar.altitude(), solar.azimuth());

                    if let Some((altitude, azimuth)) = previous {
                        let delta_altitude = (current.0 - altitude).abs();
                        let raw_azimuth = (current.1 - azimuth).abs();
                        let delta_azimuth = raw_azimuth.min(360.0 - raw_azimuth);

                        worst_altitude = worst_altitude.max(delta_altitude / 60.0);
                        worst_azimuth = worst_azimuth.max(delta_azimuth / 60.0);
                    }

                    previous = Some(current);
                }
            }
        }

        (worst_altitude, worst_azimuth)
    }

    /// The default interval must leave the applied frame within a sixth of the way
    /// from its own sun position to the next frame's.
    ///
    /// This is the reason the default is 15 seconds and not 30: at 30 the lag reaches
    /// a fifth of the tightest frame spacing, and the frame shown near every switch
    /// would visibly belong to the previous one.
    #[test]
    fn the_default_interval_keeps_the_frame_within_a_tenth_of_its_neighbour() {
        let (altitude_rate, azimuth_rate) = worst_case_solar_rates();
        let seconds = f64::from(DEFAULT_INTERVAL_SECONDS);

        // The selector weights altitude twice the azimuth: [-90°, +90°] and [0°, 360°)
        // are both normalized to 1.
        let lag = ((altitude_rate * seconds / 180.0).powi(2)
            + (azimuth_rate * seconds / 360.0).powi(2))
        .sqrt();

        assert!(
            lag <= TIGHTEST_FRAME_SPACING / 6.0,
            "a {DEFAULT_INTERVAL_SECONDS} s interval lags the sun by {lag:.4} of the \
             selection metric, more than a sixth of the tightest frame spacing \
             ({TIGHTEST_FRAME_SPACING}); rates were {altitude_rate:.5} °/s altitude and \
             {azimuth_rate:.5} °/s azimuth"
        );
    }

    /// The defaults have to be the documented ones and to pass validation: the range
    /// check lives in `validate`, so comparing the constants to each other here would
    /// only assert what the compiler already knows.
    #[test]
    fn the_default_configuration_is_accepted_by_validation() {
        let config = Config::default();

        assert_eq!(config.interval, DEFAULT_INTERVAL_SECONDS);
        assert_eq!(
            config.position_source,
            PositionSource::Automatic {
                update_on_each_calculation: false,
                cache_minutes: DEFAULT_POSITION_CACHE_MINUTES,
            }
        );
        config
            .validate()
            .expect("the default configuration has to validate");
    }

    /// The default file the daemon writes when none exists has to be readable as the
    /// configuration it describes — it is the only documentation a headless install
    /// starts with.
    #[test]
    fn the_default_configuration_round_trips() {
        let dir = std::env::temp_dir().join("dwall_config_default_roundtrip");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        ConfigWriter.write_default(&path).unwrap();
        let read_back = ConfigReader::read_from_path(&path).unwrap();

        assert_eq!(read_back, Config::default());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_default_configuration_documents_its_keys() {
        let dir = std::env::temp_dir().join("dwall_config_default_header");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        ConfigWriter.write_default(&path).unwrap();
        let contents = fs::read_to_string(&path).unwrap();

        assert!(contents.starts_with("# Dwall configuration"));
        assert!(contents.contains("--check-config"));
        assert!(contents.contains("docs/headless.md"));
        assert!(contents.contains("DWALL_LOG=debug"));

        let _ = fs::remove_dir_all(&dir);
    }

    /// The round trip through TOML, which the JSON snapshot above does not cover.
    #[test]
    fn the_default_configuration_serialises_to_toml_and_back() {
        let toml_string = toml::to_string(&Config::default()).unwrap();
        let parsed: Config = toml::from_str(&toml_string).unwrap();

        assert_eq!(parsed, Config::default());
    }

    // ── ConfigWatcher ─────────────────────────────────────────────────────────

    fn watcher(name: &str) -> (ConfigWatcher, PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("dwall_config_watcher_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        (ConfigWatcher::new(path.clone()), path, dir)
    }

    /// The daemon records the modification time before its first cycle, so the file
    /// that is already there is not a change.
    #[test]
    fn the_startup_baseline_is_not_a_change() {
        let (mut watcher, path, dir) = watcher("baseline");
        fs::write(&path, "interval = 30\n").unwrap();
        watcher.update_modified_time().unwrap();

        assert!(!watcher.has_changed().unwrap());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_watcher_reports_appeared_disappeared_and_modified() {
        let (mut watcher, path, dir) = watcher("changes");

        // Missing at startup, then created: a change.
        watcher.update_modified_time().unwrap();
        assert!(!watcher.has_changed().unwrap(), "still missing");

        fs::write(&path, "interval = 30\n").unwrap();
        assert!(watcher.has_changed().unwrap(), "appeared");

        watcher.update_modified_time().unwrap();
        assert!(!watcher.has_changed().unwrap(), "recorded");

        std::thread::sleep(std::time::Duration::from_millis(20));
        fs::write(&path, "interval = 45\n").unwrap();
        assert!(watcher.has_changed().unwrap(), "modified");

        watcher.update_modified_time().unwrap();
        fs::remove_file(&path).unwrap();
        assert!(watcher.has_changed().unwrap(), "disappeared");

        let _ = fs::remove_dir_all(&dir);
    }

    // ── Coordinate and format validation ──────────────────────────────────────

    #[test]
    fn config_validate_rejects_out_of_range_manual_coordinates() {
        let with_manual = |latitude: f64, longitude: f64| Config {
            position_source: PositionSource::Manual {
                latitude,
                longitude,
                altitude: 0.0,
            },
            ..Config::default()
        };

        for (latitude, longitude) in [(90.1, 0.0), (0.0, 180.1), (-90.1, 0.0), (0.0, -180.1)] {
            let error = with_manual(latitude, longitude)
                .validate()
                .expect_err("out-of-range manual coordinates must be refused");

            assert!(
                matches!(
                    error,
                    crate::error::DwallError::Config(ConfigError::Validation { .. })
                ),
                "the refusal has to be a validation error, got {error:?}"
            );
        }

        for (latitude, longitude) in [(90.0, 180.0), (-90.0, -180.0), (0.0, 0.0)] {
            with_manual(latitude, longitude)
                .validate()
                .expect("the inclusive coordinate bounds are valid");
        }
    }

    #[test]
    fn image_format_rejects_an_unknown_string() {
        assert_eq!(ImageFormat::Png.as_str(), "png");
        assert_eq!(ImageFormat::Jpeg.as_str(), "jpg");

        assert_eq!(
            serde_json::from_str::<ImageFormat>(r#""png""#).unwrap(),
            ImageFormat::Png
        );
        assert_eq!(
            serde_json::from_str::<ImageFormat>(r#""jpeg""#).unwrap(),
            ImageFormat::Jpeg
        );

        for unknown in [r#""webp""#, r#""JPEG""#] {
            let error = serde_json::from_str::<ImageFormat>(unknown)
                .expect_err("an unknown image format must be refused")
                .to_string();

            assert!(
                error.contains("unknown variant"),
                "{unknown} has to be refused as an unknown variant: {error}"
            );
        }
    }

    #[test]
    fn monitor_specific_wallpapers_all_variant_helpers() {
        let all = MonitorSpecificWallpapers::All("theme1".to_string());
        assert!(all.is_all());
        assert!(!all.is_empty());
        assert_eq!(all.get("any-monitor").map(String::as_str), Some("theme1"));

        let empty = MonitorSpecificWallpapers::Individual(std::collections::HashMap::new());
        assert!(!empty.is_all());
        assert!(empty.is_empty());
        assert_eq!(empty.get("monitor1"), None);

        let mut map = std::collections::HashMap::new();
        map.insert("monitor1".to_string(), "theme1".to_string());
        let individual = MonitorSpecificWallpapers::Individual(map);
        assert!(!individual.is_all());
        assert!(!individual.is_empty());
        assert_eq!(
            individual.get("monitor1").map(String::as_str),
            Some("theme1")
        );
        assert_eq!(individual.get("monitor2"), None);
    }

    #[test]
    fn wallpaper_mode_rejects_an_unknown_tag() {
        let error = serde_json::from_str::<WallpaperMode>(r#"{"mode":"shuffle"}"#)
            .expect_err("shuffle is not a wallpaper mode")
            .to_string();

        assert!(
            error.contains("unknown variant"),
            "the refusal has to name the unknown mode: {error}"
        );
    }
}
