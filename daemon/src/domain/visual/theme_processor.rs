//! Theme processing and management for visual wallpaper updates

use std::{path::Path, time::Duration};

use time::OffsetDateTime;
use tokio::{fs, time::sleep};

use crate::{
    config::Config,
    domain::{
        geography::{provider::GeographicPositionProvider, Coordinate},
        time::solar_calculator::{SolarAngle, SunPosition},
        visual::color_scheme::{determine_color_mode, set_color_mode},
    },
    infrastructure::display::wallpaper_setter::WallpaperSetter,
    DwallResult,
};

/// Comprehensive error handling for theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme does not exist")]
    NotExists,
    #[error("Missing default theme")]
    MissingDefaultTheme,
    #[error("Missing solar configuration file")]
    MissingSolarConfigFile,
    #[error("Image count does not match solar configuration")]
    ImageCountMismatch,
    #[error("Wallpaper file does not exist")]
    MissingWallpaperFile,
    #[error("No monitor-specific wallpapers found")]
    NoMonitorSpecificWallpapers,
}

/// Manages the lifecycle and processing of visual themes
pub(crate) struct ThemeProcessor<'a> {
    config: &'a Config,
    position_provider: GeographicPositionProvider<'a>,
    wallpaper_setter: WallpaperSetter,
}

impl<'a> ThemeProcessor<'a> {
    /// Creates a new ThemeProcessor instance
    pub(crate) fn new(config: &'a Config) -> DwallResult<Self> {
        debug!(
            auto_detect_color_mode = ?config.auto_detect_color_mode(),
            image_format = ?config.image_format(),
            "Initializing ThemeProcessor"
        );

        let wallpaper_setter = WallpaperSetter::new()?;

        Ok(Self {
            position_provider: GeographicPositionProvider::new(config.coordinate_source()),
            config,
            wallpaper_setter,
        })
    }

    /// Starts a continuous loop to update theme based on current position
    pub async fn start_update_loop(&self) -> DwallResult<()> {
        info!("Starting theme update loop");

        let mut last_update_time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
        let mut consecutive_failures = 0;
        const MAX_CONSECUTIVE_FAILURES: u8 = 3;

        let sleep_duration = Duration::from_secs(self.config.interval().into());

        loop {
            let current_time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
            debug!(
                last_update_time = %last_update_time,
                current_time = %current_time,
                "Beginning next theme update cycle"
            );

            // Get current position and process theme cycle
            let current_position = self.position_provider.get_current_position().await?;
            let cycle_result = self.process_theme_cycle(&current_position).await;

            match cycle_result {
                Ok(_) => {
                    debug!("Theme cycle processed successfully");
                    consecutive_failures = 0; // Reset failure counter on success
                }
                Err(e) => {
                    consecutive_failures += 1;
                    error!(
                        error = %e,
                        consecutive_failures,
                        max_failures = MAX_CONSECUTIVE_FAILURES,
                        "Failed to process theme cycle"
                    );

                    if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                        error!("Too many consecutive failures, terminating update loop");
                        break;
                    }
                }
            }

            // Check if monitor configuration has changed after processing
            let monitor_config_changed = self
                .wallpaper_setter
                .is_monitor_configuration_stale()
                .await
                .unwrap_or(false);

            if monitor_config_changed {
                info!("Monitor configuration changed, refreshing and reapplying immediately");

                // Refresh monitor list
                if let Err(e) = self.wallpaper_setter.reload_monitor_configuration().await {
                    warn!(error = %e, "Failed to reload monitor configuration");
                } else {
                    // Immediately reapply wallpaper to new monitor configuration
                    info!("Reapplying wallpaper to updated monitor configuration");
                    let reapply_result = self.process_theme_cycle(&current_position).await;

                    if let Err(e) = reapply_result {
                        error!(error = %e, "Failed to reapply wallpaper after monitor change");
                    } else {
                        debug!("Wallpaper successfully applied to new monitor configuration");
                    }
                }

                // Skip sleep and continue immediately to next cycle
                last_update_time = current_time;
                continue;
            }

            // Normal flow: sleep before next update
            debug!(
                sleep_seconds = sleep_duration.as_secs(),
                "Waiting before next theme update"
            );

            last_update_time = current_time;
            sleep(sleep_duration).await;
        }

        warn!("Theme update loop terminated");
        Ok(())
    }

    /// Process theme cycle for the current geographic position
    pub(crate) async fn process_theme_cycle(&self, position: &Coordinate) -> DwallResult<()> {
        process_theme_cycle(self.config, position, &self.wallpaper_setter).await
    }
}

/// Theme validation utilities
pub struct ThemeValidator;

impl ThemeValidator {
    /// Checks if a theme exists and has valid configuration
    pub async fn validate_theme(themes_directory: &Path, theme_id: &str) -> DwallResult<()> {
        trace!(theme_id = theme_id, "Validating theme");
        let theme_dir = themes_directory.join(theme_id);

        if !theme_dir.exists() {
            warn!(theme_id = theme_id, "Theme directory not found");
            return Err(ThemeError::NotExists.into());
        }

        let solar_angles = Self::read_solar_configuration(&theme_dir).await?;
        let image_indices: Vec<u8> = solar_angles.iter().map(|angle| angle.index).collect();

        if !Self::validate_image_files(&theme_dir, &image_indices, "jpg") {
            warn!(theme_id = theme_id, "Image validation failed for theme");
            return Err(ThemeError::ImageCountMismatch.into());
        }

        debug!(theme_id = theme_id, "Theme validation successful");
        Ok(())
    }

    /// Reads solar configuration from theme directory
    async fn read_solar_configuration(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
        let solar_config_path = theme_dir.join("solar.json");

        if !solar_config_path.exists() {
            error!(solar_config_path = %solar_config_path.display(), "Solar configuration file missing");
            return Err(ThemeError::MissingSolarConfigFile.into());
        }

        let solar_config_content = fs::read_to_string(&solar_config_path).await.map_err(|e| {
            error!(error = %e, "Failed to read solar configuration");
            e
        })?;

        let solar_angles: Vec<SolarAngle> =
            serde_json::from_str(&solar_config_content).map_err(|e| {
                error!(error = %e, "Failed to parse solar configuration JSON");
                e
            })?;

        debug!(
            solar_angles_count = solar_angles.len(),
            "Loaded solar angles from configuration"
        );
        Ok(solar_angles)
    }

    /// Validates image files in the theme directory
    fn validate_image_files(theme_dir: &Path, indices: &[u8], image_format: &str) -> bool {
        let image_dir = theme_dir.join(image_format);

        if !image_dir.is_dir() {
            warn!(image_dir = %image_dir.display(), "Image directory not found");
            return false;
        }

        let validation_result = indices.iter().all(|&index| {
            let image_filename = format!("{}.{}", index + 1, image_format);
            let image_path = image_dir.join(image_filename);

            let is_valid = image_path.exists() && image_path.is_file();
            if !is_valid {
                warn!(image_path = %image_path.display(), "Missing or invalid image");
            }
            is_valid
        });

        validation_result
    }
}

/// Load solar configuration for a specific theme directory
async fn load_solar_angles(theme_directory: &Path) -> DwallResult<Vec<SolarAngle>> {
    use std::collections::HashMap;
    use tokio::sync::OnceCell;

    // Cache solar configuration to avoid repeated reads
    static SOLAR_CACHE: OnceCell<std::sync::Mutex<HashMap<std::path::PathBuf, Vec<SolarAngle>>>> =
        OnceCell::const_new();

    let cache = SOLAR_CACHE
        .get_or_init(|| async { std::sync::Mutex::new(HashMap::new()) })
        .await;

    // Check cache
    {
        let cache_lock = cache.lock().unwrap();
        if let Some(cached_angles) = cache_lock.get(theme_directory) {
            debug!("Using cached solar configuration");
            return Ok(cached_angles.clone());
        }
    }

    let solar_config_path = theme_directory.join("solar.json");

    // Validate solar configuration file exists
    if !solar_config_path.exists() {
        error!(
            solar_config_path = %solar_config_path.display(),
            "Solar configuration file is missing"
        );
        return Err(ThemeError::MissingSolarConfigFile.into());
    }

    // Read solar configuration file
    let solar_config_content = match fs::read_to_string(&solar_config_path).await {
        Ok(content) => content,
        Err(read_error) => {
            error!(
                solar_config_path = %solar_config_path.display(),
                error = ?read_error,
                "Failed to read solar configuration file"
            );
            return Err(read_error.into());
        }
    };

    // Parse solar configuration
    let solar_angles: Vec<SolarAngle> = match serde_json::from_str(&solar_config_content) {
        Ok(angles) => angles,
        Err(parse_error) => {
            error!(
                solar_config_path = %solar_config_path.display(),
                error = ?parse_error,
                "Failed to parse solar configuration JSON"
            );
            return Err(parse_error.into());
        }
    };

    debug!(
        solar_angles_count = solar_angles.len(),
        "Successfully loaded solar configuration"
    );

    // Cache solar configuration
    {
        let mut cache_lock = cache.lock().unwrap();
        cache_lock.insert(theme_directory.to_path_buf(), solar_angles.clone());
    }

    Ok(solar_angles)
}

/// Build wallpaper path based on theme directory, image format, and image index
fn build_wallpaper_path<'a>(
    theme_directory: &Path,
    image_format: impl Into<&'a str>,
    image_index: u8,
) -> std::path::PathBuf {
    let mut path = theme_directory.to_path_buf();
    path.push(image_format.into());
    path.push(format!("{}.jpg", image_index + 1));
    path
}

/// Find the closest matching image based on solar angles and sun position
async fn find_matching_wallpaper(
    theme_directory: &Path,
    sun_position: &SunPosition,
) -> DwallResult<(u8, Vec<SolarAngle>)> {
    let solar_angles = load_solar_angles(theme_directory).await?;
    let altitude = sun_position.altitude();
    let azimuth = sun_position.azimuth();

    info!(
        altitude = altitude,
        azimuth = azimuth,
        "Calculated solar angles"
    );

    let closest_image_index = WallpaperSetter::find_closest_image(&solar_angles, altitude, azimuth)
        .ok_or_else(|| {
            error!(
                theme_directory = %theme_directory.display(),
                altitude,
                azimuth,
                "No suitable image found for current sun position"
            );
            ThemeError::ImageCountMismatch
        })?;

    Ok((closest_image_index, solar_angles))
}

/// Core theme processing function
async fn process_theme_cycle(
    config: &Config,
    geographic_position: &Coordinate,
    wallpaper_setter: &WallpaperSetter,
) -> DwallResult<()> {
    debug!(
        auto_detect_color_mode = config.auto_detect_color_mode(),
        image_format = ?config.image_format(),
        latitude = geographic_position.latitude(),
        longitude = geographic_position.longitude(),
        "Processing theme cycle with parameters"
    );

    let monitors = wallpaper_setter.list_available_monitors().await?;
    let monitor_specific_wallpapers = config.monitor_specific_wallpapers();

    let current_time = OffsetDateTime::now_utc();
    let sun_position = SunPosition::new(
        geographic_position.latitude(),
        geographic_position.longitude(),
        current_time,
    );

    let mut lock_screen_wallpaper: Option<String> = None;
    let mut any_monitor_succeeded = false;

    // Process each monitor
    for monitor_id in monitors.keys() {
        let monitor_theme_id: &str = match monitor_specific_wallpapers.get(monitor_id) {
            Some(theme_id) => theme_id.as_ref(),
            None => continue,
        };

        info!(monitor_theme_id, monitor_id, "Determined theme for monitor");

        if lock_screen_wallpaper.is_none() {
            lock_screen_wallpaper = Some(monitor_theme_id.to_string());
        }

        if let Err(e) = process_monitor_wallpaper(
            config,
            monitor_id,
            monitor_theme_id,
            &sun_position,
            wallpaper_setter,
        )
        .await
        {
            error!(
                error = %e,
                monitor_id = monitor_id,
                theme_id = monitor_theme_id,
                "Failed to process wallpaper for monitor"
            );
            continue;
        }

        any_monitor_succeeded = true;
    }

    // Update lock screen wallpaper if enabled and at least one monitor succeeded
    if config.lock_screen_wallpaper_enabled() && any_monitor_succeeded {
        if let Some(ref theme_id) = lock_screen_wallpaper {
            if let Err(e) = set_lock_screen_wallpaper(config, theme_id, &sun_position).await {
                warn!(
                    error = %e,
                    "Failed to set lock screen wallpaper, continuing with other operations"
                );
            }
        }
    }

    // Optionally update system color mode
    if config.auto_detect_color_mode() {
        let color_mode = determine_color_mode(sun_position.altitude());
        info!(
            color_mode = ?color_mode,
            "Automatically updating system color mode"
        );
        if let Err(e) = set_color_mode(color_mode) {
            warn!(
                error = %e,
                "Failed to set system color mode, continuing with other operations"
            );
        }
    }

    Ok(())
}

/// Process theme cycle for a specific monitor
async fn process_monitor_wallpaper(
    config: &Config,
    monitor_id: &str,
    theme_id: &str,
    sun_position: &SunPosition,
    wallpaper_setter: &WallpaperSetter,
) -> DwallResult<()> {
    let theme_directory = config.themes_directory().join(theme_id);
    let (closest_image_index, _) = find_matching_wallpaper(&theme_directory, sun_position).await?;
    let wallpaper_path =
        build_wallpaper_path(&theme_directory, config.image_format(), closest_image_index);

    info!(
        wallpaper_path = %wallpaper_path.display(),
        image_index = closest_image_index,
        monitor_id = monitor_id,
        "Selected wallpaper for monitor"
    );

    if !wallpaper_path.exists() {
        error!(
            wallpaper_path = %wallpaper_path.display(),
            "Wallpaper file does not exist"
        );
        return Err(ThemeError::MissingWallpaperFile.into());
    }

    wallpaper_setter
        .set_monitor_wallpaper(monitor_id, &wallpaper_path)
        .await?;

    Ok(())
}

/// Set lock screen wallpaper based on theme and sun position
async fn set_lock_screen_wallpaper(
    config: &Config,
    theme_id: &str,
    sun_position: &SunPosition,
) -> DwallResult<()> {
    let theme_directory = config.themes_directory().join(theme_id);

    match find_matching_wallpaper(&theme_directory, sun_position).await {
        Ok((closest_image_index, _)) => {
            let wallpaper_path =
                build_wallpaper_path(&theme_directory, config.image_format(), closest_image_index);

            if wallpaper_path.exists() {
                info!(
                    wallpaper_path = %wallpaper_path.display(),
                    "Setting lock screen wallpaper"
                );
                WallpaperSetter::set_lock_screen_image(&wallpaper_path)?
            } else {
                warn!(
                    wallpaper_path = %wallpaper_path.display(),
                    "Lock screen wallpaper file does not exist"
                );
            }
            Ok(())
        }
        Err(e) => {
            warn!(
                error = %e,
                theme_id = theme_id,
                "Failed to find matching wallpaper for lock screen"
            );
            Err(e)
        }
    }
}

/// Applies a theme and starts a background task for periodic wallpaper updates
pub async fn apply_theme(config: Config) -> DwallResult<()> {
    if config.monitor_specific_wallpapers().is_empty() {
        warn!("No monitor-specific wallpapers found, daemon will not be started");
        return Err(ThemeError::NoMonitorSpecificWallpapers.into());
    }

    let theme_processor = ThemeProcessor::new(&config)?;
    theme_processor.start_update_loop().await
}
