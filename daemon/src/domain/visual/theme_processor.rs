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
    config::Config,
    domain::{
        geography::{provider::GeographicPositionProvider, Position},
        time::solar_calculator::{SolarAngle, SunPosition},
        visual::color_scheme::{
            determine_color_scheme_with_hysteresis, set_color_scheme, ColorSchemeManager,
            ThresholdConfig,
        },
    },
    infrastructure::display::wallpaper_setter::WallpaperSetter,
    utils::datetime::UtcDateTime,
    DwallResult,
};

// Constants for improved code maintainability
const MAX_CONSECUTIVE_FAILURE_THRESHOLD: u8 = 3;
const SOLAR_CONFIG_FILENAME: &str = "solar.json";

/// Comprehensive error handling for solar theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeProcessingError {
    #[error("Theme directory '{0}' does not exist")]
    ThemeDirectoryNotFound(String),
    #[error("Default theme is missing or not configured")]
    DefaultThemeMissing,
    #[error("Solar configuration file 'solar.json' is missing in theme directory")]
    SolarConfigurationMissing,
    #[error("Image files do not match solar configuration: expected {expected}, found {found}")]
    ImageSolarConfigurationMismatch { expected: usize, found: usize },
    #[error("Wallpaper image file '{path}' does not exist")]
    WallpaperImageMissing { path: String },
    #[error("No monitor-specific wallpaper configurations found")]
    MonitorWallpaperConfigurationMissing,
}

/// Manages the lifecycle and processing of solar-based visual themes
pub(crate) struct SolarThemeProcessor<'a> {
    config: &'a Config,
    geographic_position_provider: GeographicPositionProvider<'a>,
    wallpaper_manager: WallpaperSetter,
}

impl<'a> SolarThemeProcessor<'a> {
    /// Creates a new SolarThemeProcessor instance with the provided configuration
    pub(crate) fn new(config: &'a Config) -> DwallResult<Self> {
        info!(
            auto_detect_color_mode = ?config.auto_detect_color_scheme(),
            image_format = ?config.image_format(),
            update_interval_seconds = config.interval(),
            "Initializing solar theme processor"
        );

        let wallpaper_manager = WallpaperSetter::new()?;

        Ok(Self {
            geographic_position_provider: GeographicPositionProvider::new(config.position_source()),
            config,
            wallpaper_manager,
        })
    }

    /// Starts a continuous loop to update wallpaper themes based on current solar position
    pub fn start_solar_update_loop(&self) -> DwallResult<()> {
        info!(
            update_interval_seconds = self.config.interval(),
            "Starting solar-based theme update loop"
        );

        let mut last_update_timestamp = UtcDateTime::now();
        let mut consecutive_failure_count = 0;

        let update_interval_duration = Duration::from_secs(self.config.interval().into());

        loop {
            let current_timestamp = UtcDateTime::now();
            debug!(
                last_update_timestamp = %last_update_timestamp,
                current_timestamp = %current_timestamp,
                "Starting solar theme update cycle"
            );

            // Get current geographic position and process solar theme cycle
            let current_geographic_position =
                self.geographic_position_provider.get_current_position()?;
            let theme_cycle_result = self.process_solar_theme_cycle(&current_geographic_position);

            match theme_cycle_result {
                Ok(_) => {
                    debug!(
                        latitude = current_geographic_position.latitude(),
                        longitude = current_geographic_position.longitude(),
                        "Solar theme cycle completed successfully"
                    );
                    consecutive_failure_count = 0; // Reset failure counter on success
                }
                Err(error) => {
                    consecutive_failure_count += 1;
                    error!(
                        error = %error,
                        consecutive_failure_count,
                        max_failure_threshold = MAX_CONSECUTIVE_FAILURE_THRESHOLD,
                        "Solar theme cycle failed"
                    );

                    if consecutive_failure_count >= MAX_CONSECUTIVE_FAILURE_THRESHOLD {
                        error!(
                            consecutive_failures = consecutive_failure_count,
                            "Maximum consecutive failures reached, terminating solar theme update loop"
                        );
                        break;
                    }
                }
            }

            // Check if monitor configuration has changed after processing
            let monitor_configuration_changed = self
                .wallpaper_manager
                .is_monitor_configuration_stale()
                .unwrap_or(false);

            if monitor_configuration_changed {
                info!(
                    "Monitor configuration change detected, refreshing and reapplying wallpapers"
                );

                // Refresh monitor list
                if let Err(reload_error) = self.wallpaper_manager.reload_monitor_configuration() {
                    warn!(
                        error = %reload_error,
                        "Failed to reload monitor configuration after change detection"
                    );
                } else {
                    // Immediately reapply wallpaper to new monitor configuration
                    info!("Reapplying solar wallpapers to updated monitor configuration");
                    let reapply_result =
                        self.process_solar_theme_cycle(&current_geographic_position);

                    if let Err(reapply_error) = reapply_result {
                        error!(
                            error = %reapply_error,
                            "Failed to reapply solar wallpapers after monitor configuration change"
                        );
                    } else {
                        debug!("Solar wallpapers successfully applied to updated monitor configuration");
                    }
                }

                // Skip sleep interval and continue immediately to next cycle
                last_update_timestamp = current_timestamp;
                continue;
            }

            // Normal flow: sleep before next update
            debug!(
                sleep_duration_seconds = update_interval_duration.as_secs(),
                "Waiting before next solar theme update cycle"
            );

            last_update_timestamp = current_timestamp;
            sleep(update_interval_duration);
        }

        warn!("Solar theme update loop terminated");
        Ok(())
    }

    /// Process solar theme cycle for the current geographic position
    pub(crate) fn process_solar_theme_cycle(
        &self,
        geographic_position: &Position,
    ) -> DwallResult<()> {
        process_solar_theme_cycle(self.config, geographic_position, &self.wallpaper_manager)
    }
}

/// Theme validation utilities for solar-based wallpaper themes
pub struct SolarThemeValidator;

impl SolarThemeValidator {
    /// Validates if a solar theme exists and has proper configuration and image files
    pub fn validate_solar_theme(
        themes_directory: &Path,
        theme_identifier: &str,
    ) -> DwallResult<()> {
        trace!(
            theme_id = theme_identifier,
            themes_directory = %themes_directory.display(),
            "Starting solar theme validation"
        );

        let theme_directory_path = themes_directory.join(theme_identifier);

        if !theme_directory_path.exists() {
            warn!(
                theme_id = theme_identifier,
                theme_path = %theme_directory_path.display(),
                "Solar theme directory not found"
            );
            return Err(
                ThemeProcessingError::ThemeDirectoryNotFound(theme_identifier.to_string()).into(),
            );
        }

        let solar_angle_configuration =
            Self::load_solar_angle_configuration(&theme_directory_path)?;
        let expected_image_indices: Vec<u8> = solar_angle_configuration
            .iter()
            .map(|angle| angle.index)
            .collect();

        if !Self::validate_theme_image_files(&theme_directory_path, &expected_image_indices, "jpg")
        {
            warn!(
                theme_id = theme_identifier,
                expected_images = expected_image_indices.len(),
                "Solar theme image validation failed"
            );
            return Err(ThemeProcessingError::ImageSolarConfigurationMismatch {
                expected: expected_image_indices.len(),
                found: 0, // We'll improve this in validate_theme_image_files
            }
            .into());
        }

        info!(
            theme_id = theme_identifier,
            solar_angles_count = solar_angle_configuration.len(),
            "Solar theme validation completed successfully"
        );
        Ok(())
    }

    /// Loads solar angle configuration from theme directory  
    fn load_solar_angle_configuration(theme_directory: &Path) -> DwallResult<Vec<SolarAngle>> {
        let solar_configuration_file_path = theme_directory.join(SOLAR_CONFIG_FILENAME);

        if !solar_configuration_file_path.exists() {
            error!(
                solar_config_path = %solar_configuration_file_path.display(),
                "Solar configuration file 'solar.json' is missing from theme directory"
            );
            return Err(ThemeProcessingError::SolarConfigurationMissing.into());
        }

        let solar_configuration_content = fs::read_to_string(&solar_configuration_file_path)
            .map_err(|io_error| {
                error!(
                    error = %io_error,
                    solar_config_path = %solar_configuration_file_path.display(),
                    "Failed to read solar configuration file"
                );
                io_error
            })?;

        let solar_angle_list: Vec<SolarAngle> = serde_json::from_str(&solar_configuration_content)
            .map_err(|json_error| {
                error!(
                    error = %json_error,
                    solar_config_path = %solar_configuration_file_path.display(),
                    "Failed to parse solar configuration JSON"
                );
                json_error
            })?;

        debug!(
            solar_angles_count = solar_angle_list.len(),
            solar_config_path = %solar_configuration_file_path.display(),
            "Successfully loaded solar angle configuration"
        );
        Ok(solar_angle_list)
    }

    /// Validates that all required image files exist for the theme's solar configuration
    fn validate_theme_image_files(
        theme_directory: &Path,
        expected_image_indices: &[u8],
        image_file_format: &str,
    ) -> bool {
        let images_directory_path = theme_directory.join(image_file_format);

        if !images_directory_path.is_dir() {
            warn!(
                images_directory = %images_directory_path.display(),
                "Theme images directory not found or is not a directory"
            );
            return false;
        }

        let mut missing_images_count = 0;
        let validation_successful = expected_image_indices.iter().all(|&image_index| {
            let image_filename = format!("{}.{}", image_index + 1, image_file_format);
            let image_file_path = images_directory_path.join(image_filename);

            let image_exists = image_file_path.exists() && image_file_path.is_file();
            if !image_exists {
                missing_images_count += 1;
                warn!(
                    image_path = %image_file_path.display(),
                    image_index = image_index,
                    "Required theme image file is missing"
                );
            }
            image_exists
        });

        if !validation_successful {
            error!(
                missing_images = missing_images_count,
                total_expected = expected_image_indices.len(),
                "Theme image validation failed due to missing files"
            );
        }

        validation_successful
    }
}

thread_local! {
    /// Cache solar configuration to avoid repeated reads
    static SOLAR_CACHE: RefCell<HashMap<PathBuf, Vec<SolarAngle>>> =
        RefCell::new(HashMap::new());
}

/// Load solar configuration for a specific theme directory
fn load_cached_solar_angles(theme_directory: &Path) -> DwallResult<Vec<SolarAngle>> {
    let theme_directory = theme_directory.canonicalize()?;
    debug!(path = %theme_directory.display(), "Loading solar configuration from canonical and absolute path");

    // Check cache
    {
        if let Some(cached_angles) =
            SOLAR_CACHE.with(|cache| cache.borrow().get(&theme_directory).cloned())
        {
            debug!("Using cached solar configuration");
            return Ok(cached_angles);
        }
    }
    let solar_config_path = theme_directory.join("solar.json");
    // Validate solar configuration file exists
    if !solar_config_path.exists() {
        error!(
            solar_config_path = %solar_config_path.display(),
            "Solar configuration file is missing"
        );
        return Err(ThemeProcessingError::SolarConfigurationMissing.into());
    }
    // Read solar configuration file
    let solar_config_content = match fs::read_to_string(&solar_config_path) {
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
    SOLAR_CACHE.with(|cache| {
        cache
            .borrow_mut()
            .insert(theme_directory.to_path_buf(), solar_angles.clone());
    });

    Ok(solar_angles)
}

/// Build wallpaper file path based on theme directory, image format, and solar image index
fn construct_wallpaper_file_path<'a>(
    theme_directory_path: &Path,
    image_file_format: impl Into<&'a str>,
    solar_image_index: u8,
) -> PathBuf {
    let image_format_str = image_file_format.into();
    let mut wallpaper_path = theme_directory_path.to_path_buf();
    wallpaper_path.push(image_format_str);
    wallpaper_path.push(format!("{}.{}", solar_image_index + 1, image_format_str));
    wallpaper_path
}

/// Find the wallpaper image that best matches the current solar position
fn find_optimal_solar_wallpaper(
    theme_directory_path: &Path,
    current_sun_position: &SunPosition,
) -> DwallResult<(u8, Vec<SolarAngle>)> {
    let solar_angle_configuration = load_cached_solar_angles(theme_directory_path)?;
    let sun_altitude_degrees = current_sun_position.altitude();
    let sun_azimuth_degrees = current_sun_position.azimuth();

    debug!(
        sun_altitude = sun_altitude_degrees,
        sun_azimuth = sun_azimuth_degrees,
        theme_directory = %theme_directory_path.display(),
        "Calculated current solar position for wallpaper selection"
    );

    let optimal_image_index = WallpaperSetter::find_closest_image(
        &solar_angle_configuration,
        sun_altitude_degrees,
        sun_azimuth_degrees,
    )
    .ok_or_else(|| {
        error!(
            theme_directory = %theme_directory_path.display(),
            sun_altitude = sun_altitude_degrees,
            sun_azimuth = sun_azimuth_degrees,
            solar_angles_available = solar_angle_configuration.len(),
            "No suitable wallpaper image found for current solar position"
        );
        ThemeProcessingError::ImageSolarConfigurationMismatch {
            expected: solar_angle_configuration.len(),
            found: 0,
        }
    })?;

    info!(
        optimal_image_index,
        sun_altitude = sun_altitude_degrees,
        sun_azimuth = sun_azimuth_degrees,
        "Selected optimal wallpaper image for current solar position"
    );

    Ok((optimal_image_index, solar_angle_configuration))
}

/// Core solar theme processing function that updates wallpapers for all monitors
fn process_solar_theme_cycle(
    configuration: &Config,
    current_geographic_position: &Position,
    wallpaper_manager: &WallpaperSetter,
) -> DwallResult<()> {
    debug!(
        auto_detect_color_mode = configuration.auto_detect_color_scheme(),
        image_format = ?configuration.image_format(),
        latitude = current_geographic_position.latitude(),
        longitude = current_geographic_position.longitude(),
        "Starting solar theme processing cycle"
    );

    let available_monitors = wallpaper_manager.list_available_monitors()?;
    let monitor_theme_configurations = configuration.monitor_specific_wallpapers();

    let current_utc_time = UtcDateTime::now();
    let current_sun_position = SunPosition::new(
        current_geographic_position.latitude(),
        current_geographic_position.longitude(),
        current_utc_time,
    );

    let mut lock_screen_theme_identifier: Option<String> = None;
    let mut successful_monitor_count = 0;

    // Process wallpaper update for each configured monitor
    for monitor_identifier in available_monitors.keys() {
        let assigned_theme_id: &str = match monitor_theme_configurations.get(monitor_identifier) {
            Some(theme_id) => theme_id.as_ref(),
            None => {
                debug!(
                    monitor_id = monitor_identifier,
                    "No theme configuration found for monitor, skipping"
                );
                continue;
            }
        };

        info!(
            monitor_id = monitor_identifier,
            assigned_theme = assigned_theme_id,
            "Processing solar wallpaper for monitor"
        );

        // Use the first successfully configured theme for lock screen
        if lock_screen_theme_identifier.is_none() {
            lock_screen_theme_identifier = Some(assigned_theme_id.to_string());
        }

        if let Err(processing_error) = update_monitor_solar_wallpaper(
            configuration,
            monitor_identifier,
            assigned_theme_id,
            &current_sun_position,
            wallpaper_manager,
        ) {
            error!(
                error = %processing_error,
                monitor_id = monitor_identifier,
                theme_id = assigned_theme_id,
                "Failed to update solar wallpaper for monitor"
            );
            continue;
        }

        successful_monitor_count += 1;
    }

    // Update lock screen wallpaper if enabled and at least one monitor was successful
    if configuration.lock_screen_wallpaper_enabled() && successful_monitor_count > 0 {
        if let Some(ref lock_screen_theme_id) = lock_screen_theme_identifier {
            if let Err(lock_screen_error) = apply_lock_screen_solar_wallpaper(
                configuration,
                lock_screen_theme_id,
                &current_sun_position,
            ) {
                warn!(
                    error = %lock_screen_error,
                    theme_id = lock_screen_theme_id,
                    "Failed to apply solar wallpaper to lock screen, continuing with other operations"
                );
            }
        }
    }

    // Optionally update system color scheme based on solar position
    if configuration.auto_detect_color_scheme() {
        let current_color_scheme = ColorSchemeManager::get_current_scheme()?;
        let solar_based_color_scheme = determine_color_scheme_with_hysteresis(
            current_sun_position.altitude(),
            &current_color_scheme,
            &ThresholdConfig::from_location(current_geographic_position),
        );
        info!(
            color_scheme = ?solar_based_color_scheme,
            sun_altitude = current_sun_position.altitude(),
            "Automatically updating system color scheme based on solar position"
        );
        if let Err(color_scheme_error) = set_color_scheme(solar_based_color_scheme) {
            warn!(
                error = %color_scheme_error,
                "Failed to update system color scheme, continuing with other operations"
            );
        }
    }

    info!(
        successful_monitors = successful_monitor_count,
        total_monitors = available_monitors.len(),
        "Solar theme processing cycle completed"
    );

    Ok(())
}

/// Update solar wallpaper for a specific monitor based on current sun position
fn update_monitor_solar_wallpaper(
    configuration: &Config,
    monitor_identifier: &str,
    theme_identifier: &str,
    current_sun_position: &SunPosition,
    wallpaper_manager: &WallpaperSetter,
) -> DwallResult<()> {
    let theme_directory_path = configuration.themes_directory().join(theme_identifier);
    let (optimal_image_index, _) =
        find_optimal_solar_wallpaper(&theme_directory_path, current_sun_position)?;
    let wallpaper_file_path = construct_wallpaper_file_path(
        &theme_directory_path,
        configuration.image_format(),
        optimal_image_index,
    );

    info!(
        wallpaper_path = %wallpaper_file_path.display(),
        image_index = optimal_image_index,
        monitor_id = monitor_identifier,
        theme_id = theme_identifier,
        "Selected optimal solar wallpaper for monitor"
    );

    if !wallpaper_file_path.exists() {
        error!(
            wallpaper_path = %wallpaper_file_path.display(),
            theme_id = theme_identifier,
            "Selected wallpaper image file does not exist"
        );
        return Err(ThemeProcessingError::WallpaperImageMissing {
            path: wallpaper_file_path.display().to_string(),
        }
        .into());
    }

    wallpaper_manager.set_monitor_wallpaper(monitor_identifier, &wallpaper_file_path)?;

    debug!(
        monitor_id = monitor_identifier,
        wallpaper_path = %wallpaper_file_path.display(),
        "Successfully applied solar wallpaper to monitor"
    );

    Ok(())
}

/// Apply solar wallpaper to lock screen based on theme and current sun position
fn apply_lock_screen_solar_wallpaper(
    configuration: &Config,
    theme_identifier: &str,
    current_sun_position: &SunPosition,
) -> DwallResult<()> {
    let theme_directory_path = configuration.themes_directory().join(theme_identifier);

    match find_optimal_solar_wallpaper(&theme_directory_path, current_sun_position) {
        Ok((optimal_image_index, _)) => {
            let wallpaper_file_path = construct_wallpaper_file_path(
                &theme_directory_path,
                configuration.image_format(),
                optimal_image_index,
            );

            if wallpaper_file_path.exists() {
                info!(
                    wallpaper_path = %wallpaper_file_path.display(),
                    theme_id = theme_identifier,
                    "Applying optimal solar wallpaper to lock screen"
                );
                WallpaperSetter::set_lock_screen_image(&wallpaper_file_path)?;
            } else {
                warn!(
                    wallpaper_path = %wallpaper_file_path.display(),
                    theme_id = theme_identifier,
                    "Optimal lock screen wallpaper file does not exist"
                );
                return Err(ThemeProcessingError::WallpaperImageMissing {
                    path: wallpaper_file_path.display().to_string(),
                }
                .into());
            }
            Ok(())
        }
        Err(wallpaper_error) => {
            warn!(
                error = %wallpaper_error,
                theme_id = theme_identifier,
                "Failed to find optimal solar wallpaper for lock screen"
            );
            Err(wallpaper_error)
        }
    }
}

/// Applies a solar theme and starts background processing for periodic wallpaper updates
pub async fn apply_solar_theme(configuration: Config) -> DwallResult<()> {
    if configuration.monitor_specific_wallpapers().is_empty() {
        warn!("No monitor-specific wallpaper configurations found, solar theme daemon will not be started");
        return Err(ThemeProcessingError::MonitorWallpaperConfigurationMissing.into());
    }

    info!(
        configured_monitors = ?configuration.monitor_specific_wallpapers(),
        "Starting solar theme processor with monitor configurations"
    );

    let solar_theme_processor = SolarThemeProcessor::new(&configuration)?;
    solar_theme_processor.start_solar_update_loop()
}
