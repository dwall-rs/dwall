//! Solar-based theme processing and management for dynamic wallpaper updates

use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    thread::sleep,
    time::Duration,
};

use crate::{
    DwallResult,
    config::Config,
    domain::{
        geography::{Position, provider::GeographicPositionProvider},
        time::{
            solar_calculator::{SolarAngle, SunPosition},
            solar_transitions::{PolarState, SolarTransitions},
        },
        visual::color_scheme::{
            SwitchSchedule, determine_color_scheme_by_schedule, set_color_scheme,
        },
    },
    infrastructure::display::wallpaper_setter::WallpaperSetter,
    utils::datetime::UtcDateTime,
};

const MAX_CONSECUTIVE_FAILURES: u8 = 3;
const SOLAR_CONFIG_FILE: &str = "solar.json";

/// Comprehensive error handling for solar theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme directory '{0}' does not exist")]
    DirectoryNotFound(String),
    #[error("Default theme is missing or not configured")]
    DefaultThemeMissing,
    #[error("Solar configuration file 'solar.json' is missing in theme directory")]
    SolarConfigMissing,
    #[error("Image files do not match solar configuration: expected {expected}, found {found}")]
    ImageConfigMismatch { expected: usize, found: usize },
    #[error("Wallpaper image file '{path}' does not exist")]
    WallpaperMissing { path: String },
    #[error("No monitor-specific wallpaper configurations found")]
    NoMonitorConfig,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScheduleCacheKey {
    date: u32, // yyyymmdd
    // 位置精度截断到小数点后 3 位（约 111 米），
    // 避免 GPS 抖动导致每次 tick 都重算
    lat_i32: i32, // latitude  × 1000 as i32
    lon_i32: i32, // longitude × 1000 as i32
}

impl ScheduleCacheKey {
    fn from(now: UtcDateTime, position: &Position) -> Self {
        let (y, m, d, ..) = now.ymd_hms();
        Self {
            date: y as u32 * 10000 + m as u8 as u32 * 100 + d as u32,
            lat_i32: (position.latitude() * 1000.0).round() as i32,
            lon_i32: (position.longitude() * 1000.0).round() as i32,
        }
    }
}

// 缓存当天的切换计划，避免每分钟重算
struct CachedSchedule {
    key: ScheduleCacheKey,
    schedule: SwitchSchedule,
    polar_state: PolarState,
}

/// Manages the lifecycle and processing of solar-based visual themes
pub(crate) struct ThemeProcessor<'a> {
    config: &'a Config,
    position_provider: GeographicPositionProvider<'a>,
    wallpaper_setter: WallpaperSetter,
    cached_schedule: RefCell<Option<CachedSchedule>>,
}

impl<'a> ThemeProcessor<'a> {
    /// Creates a new `ThemeProcessor` with the provided configuration
    pub(crate) fn new(config: &'a Config) -> DwallResult<Self> {
        info!(
            auto_detect_color_mode = ?config.auto_detect_color_scheme(),
            image_format = ?config.image_format(),
            update_interval_seconds = config.interval(),
            "Initializing solar theme processor"
        );

        Ok(Self {
            position_provider: GeographicPositionProvider::new(config.position_source()),
            wallpaper_setter: WallpaperSetter::new()?,
            config,
            cached_schedule: RefCell::new(None),
        })
    }

    /// Returns the update interval in seconds
    pub(crate) fn update_interval(&self) -> u16 {
        self.config.interval()
    }

    /// Runs a single solar theme update cycle.
    ///
    /// Returns `true` if the cycle succeeded.
    pub(crate) fn run_once(&self) -> DwallResult<bool> {
        let position = self.position_provider.get_current_position()?;

        let now = UtcDateTime::now();

        let result = self.process_cycle(&position, now);

        if let Err(ref e) = result {
            error!(error = %e, "Solar theme cycle failed");
        }

        result.map(|_| true)
    }

    /// Checks if monitor configuration has changed and reloads if necessary.
    ///
    /// Returns `true` if the configuration was reloaded.
    pub(crate) fn reload_if_monitors_changed(&self) -> bool {
        let changed = self
            .wallpaper_setter
            .is_monitor_configuration_stale()
            .unwrap_or(false);

        if !changed {
            return false;
        }

        info!("Monitor configuration change detected, refreshing and reapplying wallpapers");

        if let Err(e) = self.wallpaper_setter.reload_monitor_configuration() {
            warn!(error = %e, "Failed to reload monitor configuration");
            return false;
        }

        if let Ok(position) = self.position_provider.get_current_position() {
            if let Err(e) = self.process_cycle(&position, UtcDateTime::now()) {
                error!(error = %e, "Failed to reapply wallpapers after monitor configuration change");
            } else {
                debug!("Wallpapers successfully applied to updated monitor configuration");
            }
        }

        true
    }

    /// Starts a continuous loop to update wallpaper themes based on current solar position
    pub fn start_update_loop(&self) -> DwallResult<()> {
        info!(
            update_interval_seconds = self.config.interval(),
            "Starting solar-based theme update loop"
        );

        let mut last_update = UtcDateTime::now();
        let mut failure_count = 0u8;
        let interval = Duration::from_secs(self.config.interval().into());

        loop {
            let now = UtcDateTime::now();
            debug!(
                last_update = %last_update,
                current = %now,
                "Starting solar theme update cycle"
            );

            let position = self.position_provider.get_current_position()?;

            match self.process_cycle(&position, now) {
                Ok(_) => {
                    debug!(
                        latitude = position.latitude(),
                        longitude = position.longitude(),
                        "Solar theme cycle completed successfully"
                    );
                    failure_count = 0;
                }
                Err(e) => {
                    failure_count += 1;
                    error!(
                        error = %e,
                        failure_count,
                        max_failures = MAX_CONSECUTIVE_FAILURES,
                        "Solar theme cycle failed"
                    );

                    if failure_count >= MAX_CONSECUTIVE_FAILURES {
                        error!(
                            failure_count,
                            "Maximum consecutive failures reached, terminating update loop"
                        );
                        break;
                    }
                }
            }

            let monitors_changed = self
                .wallpaper_setter
                .is_monitor_configuration_stale()
                .unwrap_or(false);

            if monitors_changed {
                info!(
                    "Monitor configuration change detected, refreshing and reapplying wallpapers"
                );

                if let Err(e) = self.wallpaper_setter.reload_monitor_configuration() {
                    warn!(error = %e, "Failed to reload monitor configuration after change detection");
                } else {
                    info!("Reapplying wallpapers to updated monitor configuration");
                    if let Err(e) = self.process_cycle(&position, now) {
                        error!(error = %e, "Failed to reapply wallpapers after monitor configuration change");
                    } else {
                        debug!("Wallpapers successfully applied to updated monitor configuration");
                    }
                }

                last_update = now;
                continue;
            }

            debug!(
                sleep_seconds = interval.as_secs(),
                "Waiting before next solar theme update cycle"
            );

            last_update = now;
            sleep(interval);
        }

        warn!("Solar theme update loop terminated");
        Ok(())
    }

    /// Processes one solar theme cycle for the given geographic position
    pub(crate) fn process_cycle(&self, position: &Position, now: UtcDateTime) -> DwallResult<()> {
        self.run_theme_cycle(position, now)
    }

    /// Core cycle function: updates wallpapers for all monitors and optionally the lock screen
    fn run_theme_cycle(&self, position: &Position, now: UtcDateTime) -> DwallResult<()> {
        debug!(
            auto_detect_color_mode = self.config.auto_detect_color_scheme(),
            image_format = ?self.config.image_format(),
            latitude = position.latitude(),
            longitude = position.longitude(),
            "Starting solar theme processing cycle"
        );

        let monitors = self.wallpaper_setter.list_available_monitors()?;
        let monitor_themes = self.config.monitor_specific_wallpapers();
        let sun = SunPosition::new(position.latitude(), position.longitude(), now);

        let mut lock_screen_theme: Option<String> = None;
        let mut success_count = 0usize;

        for monitor_id in monitors.keys() {
            let theme_id = match monitor_themes.get(monitor_id) {
                Some(id) => id,
                None => {
                    debug!(
                        monitor_id,
                        "No theme configuration found for monitor, skipping"
                    );
                    continue;
                }
            };

            info!(
                monitor_id,
                theme_id, "Processing solar wallpaper for monitor"
            );

            if lock_screen_theme.is_none() {
                lock_screen_theme = Some(theme_id.to_string());
            }

            if let Err(e) = set_monitor_wallpaper(
                self.config,
                monitor_id,
                theme_id,
                &sun,
                &self.wallpaper_setter,
            ) {
                error!(error = %e, monitor_id, theme_id, "Failed to update solar wallpaper for monitor");
                continue;
            }

            success_count += 1;
        }

        if self.config.lock_screen_wallpaper_enabled()
            && success_count > 0
            && let Some(ref theme_id) = lock_screen_theme
            && let Err(e) = set_lock_screen_wallpaper(self.config, theme_id, &sun)
        {
            warn!(
                error = %e,
                theme_id,
                "Failed to apply solar wallpaper to lock screen, continuing"
            );
        }

        if self.config.auto_detect_color_scheme() {
            let cache_key = ScheduleCacheKey::from(now, position);

            if self
                .cached_schedule
                .borrow()
                .as_ref()
                .is_none_or(|c| c.key != cache_key)
            {
                let day_start = now.start_of_day(); // 需要在 UtcDateTime 上实现此方法
                let transitions = SolarTransitions::calculate(position, day_start);
                let schedule = SwitchSchedule::from_transitions(
                    &transitions,
                    0,     // 日落即切 Dark
                    -1800, // 日出前 30 分钟切 Light
                );

                *self.cached_schedule.borrow_mut() = Some(CachedSchedule {
                    key: cache_key,
                    schedule,
                    polar_state: transitions.polar_state,
                });
            }

            let borrow = self.cached_schedule.borrow();
            let cached = borrow.as_ref().unwrap();

            let target = determine_color_scheme_by_schedule(
                now.timestamp(),
                &cached.schedule,
                cached.polar_state,
            );

            info!(
                scheme = ?target,
                altitude = sun.altitude(),
                "Updating system color scheme based on solar position"
            );
            if let Err(e) = set_color_scheme(target) {
                warn!(error = %e, "Failed to update system color scheme, continuing");
            }
        }

        info!(
            success_count,
            total = monitors.len(),
            "Solar theme processing cycle completed"
        );

        Ok(())
    }
}

/// Theme validation utilities for solar-based wallpaper themes
pub struct ThemeValidator;

impl ThemeValidator {
    /// Validates that a solar theme exists and has proper configuration and image files
    pub fn validate(themes_dir: &Path, theme_id: &str) -> DwallResult<()> {
        trace!(
            theme_id,
            themes_dir = %themes_dir.display(),
            "Starting solar theme validation"
        );

        let theme_dir = themes_dir.join(theme_id);

        if !theme_dir.exists() {
            warn!(
                theme_id,
                theme_dir = %theme_dir.display(),
                "Solar theme directory not found"
            );
            return Err(ThemeError::DirectoryNotFound(theme_id.to_string()).into());
        }

        let solar_angles = Self::load_solar_angles(&theme_dir)?;
        let expected_indices: Vec<u8> = solar_angles.iter().map(|a| a.index).collect();

        if !Self::validate_images(&theme_dir, &expected_indices, "jpg") {
            warn!(
                theme_id,
                expected_images = expected_indices.len(),
                "Solar theme image validation failed"
            );
            return Err(ThemeError::ImageConfigMismatch {
                expected: expected_indices.len(),
                found: 0,
            }
            .into());
        }

        info!(
            theme_id,
            solar_angles_count = solar_angles.len(),
            "Solar theme validation completed successfully"
        );
        Ok(())
    }

    /// Loads solar angle configuration from a theme directory
    fn load_solar_angles(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
        let config_path = theme_dir.join(SOLAR_CONFIG_FILE);

        if !config_path.exists() {
            error!(
                config_path = %config_path.display(),
                "Solar configuration file 'solar.json' is missing from theme directory"
            );
            return Err(ThemeError::SolarConfigMissing.into());
        }

        let content = fs::read_to_string(&config_path).map_err(|e| {
            error!(error = %e, config_path = %config_path.display(), "Failed to read solar configuration file");
            e
        })?;

        let angles: Vec<SolarAngle> = serde_json::from_str(&content).map_err(|e| {
            error!(error = %e, config_path = %config_path.display(), "Failed to parse solar configuration JSON");
            e
        })?;

        debug!(
            count = angles.len(),
            config_path = %config_path.display(),
            "Successfully loaded solar angle configuration"
        );
        Ok(angles)
    }

    /// Validates that all required image files exist for the theme's solar configuration
    fn validate_images(theme_dir: &Path, indices: &[u8], format: &str) -> bool {
        let images_dir = theme_dir.join(format);

        if !images_dir.is_dir() {
            warn!(
                images_dir = %images_dir.display(),
                "Theme images directory not found or is not a directory"
            );
            return false;
        }

        let mut missing = 0usize;
        let valid = indices.iter().all(|&idx| {
            let filename = format!("{}.{}", idx + 1, format);
            let path = images_dir.join(&filename);
            let exists = path.exists() && path.is_file();
            if !exists {
                missing += 1;
                warn!(image_path = %path.display(), index = idx, "Required theme image file is missing");
            }
            exists
        });

        if !valid {
            error!(
                missing,
                total = indices.len(),
                "Theme image validation failed due to missing files"
            );
        }

        valid
    }
}

thread_local! {
    /// Cache solar configuration to avoid repeated disk reads
    static SOLAR_CACHE: RefCell<HashMap<PathBuf, Vec<SolarAngle>>> =
        RefCell::new(HashMap::new());
}

/// Load solar angles for a theme directory, using an in-memory cache
fn load_solar_angles_cached(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
    let canonical = theme_dir.canonicalize()?;
    debug!(path = %canonical.display(), "Loading solar configuration");

    if let Some(cached) = SOLAR_CACHE.with(|c| c.borrow().get(&canonical).cloned()) {
        debug!("Using cached solar configuration");
        return Ok(cached);
    }

    let config_path = canonical.join(SOLAR_CONFIG_FILE);

    if !config_path.exists() {
        error!(config_path = %config_path.display(), "Solar configuration file is missing");
        return Err(ThemeError::SolarConfigMissing.into());
    }

    let content = fs::read_to_string(&config_path).map_err(|e| {
        error!(config_path = %config_path.display(), error = ?e, "Failed to read solar configuration file");
        e
    })?;

    let angles: Vec<SolarAngle> = serde_json::from_str(&content).map_err(|e| {
        error!(config_path = %config_path.display(), error = ?e, "Failed to parse solar configuration JSON");
        e
    })?;

    debug!(
        count = angles.len(),
        "Successfully loaded solar configuration"
    );

    SOLAR_CACHE.with(|c| {
        c.borrow_mut().insert(canonical, angles.clone());
    });

    Ok(angles)
}

/// Build the wallpaper file path from a theme directory, image format, and solar image index
fn wallpaper_path<'a>(theme_dir: &Path, format: impl Into<&'a str>, index: u8) -> PathBuf {
    let fmt = format.into();
    theme_dir.join(fmt).join(format!("{}.{}", index + 1, fmt))
}

/// Find the wallpaper image that best matches the current solar position
fn best_wallpaper_for_sun(
    theme_dir: &Path,
    sun: &SunPosition,
) -> DwallResult<(u8, Vec<SolarAngle>)> {
    let angles = load_solar_angles_cached(theme_dir)?;
    let altitude = sun.altitude();
    let azimuth = sun.azimuth();

    debug!(
        altitude,
        azimuth,
        theme_dir = %theme_dir.display(),
        "Selecting wallpaper for solar position"
    );

    let index =
        WallpaperSetter::find_closest_image(&angles, altitude, azimuth).ok_or_else(|| {
            error!(
                theme_dir = %theme_dir.display(),
                altitude,
                azimuth,
                available = angles.len(),
                "No suitable wallpaper found for current solar position"
            );
            ThemeError::ImageConfigMismatch {
                expected: angles.len(),
                found: 0,
            }
        })?;

    info!(
        index,
        altitude, azimuth, "Selected optimal wallpaper for solar position"
    );

    Ok((index, angles))
}

/// Apply the optimal solar wallpaper to a specific monitor
fn set_monitor_wallpaper(
    config: &Config,
    monitor_id: &str,
    theme_id: &str,
    sun: &SunPosition,
    setter: &WallpaperSetter,
) -> DwallResult<()> {
    let theme_dir = config.themes_directory().join(theme_id);
    let (index, _) = best_wallpaper_for_sun(&theme_dir, sun)?;
    let path = wallpaper_path(&theme_dir, config.image_format(), index);

    info!(
        path = %path.display(),
        index,
        monitor_id,
        theme_id,
        "Selected optimal wallpaper for monitor"
    );

    if !path.exists() {
        error!(path = %path.display(), theme_id, "Wallpaper image file does not exist");
        return Err(ThemeError::WallpaperMissing {
            path: path.display().to_string(),
        }
        .into());
    }

    setter.set_monitor_wallpaper(monitor_id, &path)?;

    debug!(monitor_id, path = %path.display(), "Successfully applied wallpaper to monitor");

    Ok(())
}

/// Apply the optimal solar wallpaper to the lock screen
fn set_lock_screen_wallpaper(
    config: &Config,
    theme_id: &str,
    sun: &SunPosition,
) -> DwallResult<()> {
    let theme_dir = config.themes_directory().join(theme_id);

    match best_wallpaper_for_sun(&theme_dir, sun) {
        Ok((index, _)) => {
            let path = wallpaper_path(&theme_dir, config.image_format(), index);

            if path.exists() {
                info!(path = %path.display(), theme_id, "Applying wallpaper to lock screen");
                WallpaperSetter::set_lock_screen_image(&path)?;
            } else {
                warn!(path = %path.display(), theme_id, "Lock screen wallpaper file does not exist");
                return Err(ThemeError::WallpaperMissing {
                    path: path.display().to_string(),
                }
                .into());
            }
            Ok(())
        }
        Err(e) => {
            warn!(error = %e, theme_id, "Failed to find optimal wallpaper for lock screen");
            Err(e)
        }
    }
}

/// Applies a solar theme and starts background processing for periodic wallpaper updates
pub async fn apply_solar_theme(config: Config) -> DwallResult<()> {
    if config.monitor_specific_wallpapers().is_empty() {
        warn!(
            "No monitor-specific wallpaper configurations found, solar theme daemon will not be started"
        );
        return Err(ThemeError::NoMonitorConfig.into());
    }

    info!(
        monitors = ?config.monitor_specific_wallpapers(),
        "Starting solar theme processor with monitor configurations"
    );

    ThemeProcessor::new(&config)?.start_update_loop()
}
