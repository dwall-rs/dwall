use std::{path::Path, time::Duration};

use time::OffsetDateTime;
use tokio::{fs, time::sleep};

use crate::{
    color_mode::{determine_color_mode, set_color_mode},
    config::Config,
    position::{Position, PositionManager},
    solar::{SolarAngle, SunPosition},
    theme::{manager::WallpaperManager, ThemeError},
    DwallResult,
};

/// Manages the lifecycle and processing of a specific theme
pub struct ThemeProcessor {
    /// Configuration settings for theme processing
    config: Config,
    /// Manages geographic position tracking
    position_manager: PositionManager,
    wallpaper_manager: WallpaperManager,
}

impl ThemeProcessor {
    /// Creates a new ThemeProcessor instance
    pub fn new(config: &Config) -> DwallResult<Self> {
        debug!(
            auto_detect_color_mode = ?config.auto_detect_color_mode(),
            image_format = ?config.image_format(),
            "Initializing ThemeProcessor for theme with configuration"
        );

        let wallpaper_manager = WallpaperManager::new()?;

        Ok(Self {
            position_manager: PositionManager::new(config.coordinate_source().clone()),
            config: config.clone(),
            wallpaper_manager,
        })
    }

    /// Starts a continuous loop to update theme based on current position
    pub async fn start_update_loop(&self) -> DwallResult<()> {
        info!("Starting theme update loop");

        let mut last_update_time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
        let mut consecutive_failures = 0;
        const MAX_CONSECUTIVE_FAILURES: u8 = 3;

        loop {
            let current_time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
            debug!(
                last_update_time = %last_update_time,
                current_time = %current_time,
                "Beginning next theme update cycle"
            );

            // Get current position and process theme cycle
            let current_position = self.position_manager.get_current_position().await?;
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

                    // Only terminate loop after multiple consecutive failures
                    if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                        error!("Too many consecutive failures, terminating update loop");
                        break;
                    }
                }
            }

            // Sleep for configured interval before next update
            let sleep_duration = Duration::from_secs(self.config.interval().into());
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
    async fn process_theme_cycle(&self, position: &Position) -> DwallResult<()> {
        process_theme_cycle(&self.config, position, &self.wallpaper_manager).await
    }
}

/// Load solar configuration for a specific theme directory
async fn load_solar_angles(theme_directory: &Path) -> DwallResult<Vec<SolarAngle>> {
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

    Ok(solar_angles)
}

/// Build wallpaper path based on theme directory, image format, and image index
fn build_wallpaper_path<'a>(
    theme_directory: &Path,
    image_format: impl Into<&'a str>,
    image_index: u8,
) -> std::path::PathBuf {
    theme_directory
        .join(image_format.into())
        .join(format!("{}.jpg", image_index + 1))
}

/// Find the closest matching image based on solar angles and sun position
async fn find_matching_wallpaper(
    theme_directory: &Path,
    sun_position: &SunPosition,
) -> DwallResult<(u8, Vec<SolarAngle>)> {
    // Load solar angles for the theme
    let solar_angles = load_solar_angles(theme_directory).await?;

    let altitude = sun_position.altitude();
    let azimuth = sun_position.azimuth();
    info!(
        altitude = altitude,
        azimuth = azimuth,
        "Calculated solar angles"
    );

    // Find the closest matching image
    let closest_image_index =
        WallpaperManager::find_closest_image(&solar_angles, altitude, azimuth).ok_or_else(
            || {
                error!(
                    theme_directory = %theme_directory.display(),
                    altitude,
                    azimuth,
                    "No suitable image found for current sun position"
                );
                ThemeError::ImageCountMismatch
            },
        )?;

    Ok((closest_image_index, solar_angles))
}

/// Process theme cycle for a specific monitor
async fn process_monitor_wallpaper(
    config: &Config,
    monitor_id: &str,
    theme_id: &str,
    sun_position: &SunPosition,
    wallpaper_manager: &WallpaperManager,
) -> DwallResult<()> {
    let theme_directory = config.themes_directory().join(theme_id);

    // Find matching wallpaper based on sun position
    let (closest_image_index, _) = find_matching_wallpaper(&theme_directory, sun_position).await?;

    // Construct wallpaper path
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

    // Set wallpaper for the specific monitor
    wallpaper_manager
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

    // Find matching wallpaper based on sun position
    match find_matching_wallpaper(&theme_directory, sun_position).await {
        Ok((closest_image_index, _)) => {
            let wallpaper_path =
                build_wallpaper_path(&theme_directory, config.image_format(), closest_image_index);

            if wallpaper_path.exists() {
                info!(
                    wallpaper_path = %wallpaper_path.display(),
                    "Setting lock screen wallpaper"
                );
                WallpaperManager::set_lock_screen_image(&wallpaper_path)?
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

/// Core theme processing function
async fn process_theme_cycle(
    config: &Config,
    geographic_position: &Position,
    wallpaper_manager: &WallpaperManager,
) -> DwallResult<()> {
    debug!(
        auto_detect_color_mode = config.auto_detect_color_mode(),
        image_format = ?config.image_format(),
        latitude = geographic_position.latitude(),
        longitude = geographic_position.longitude(),
        "Processing theme cycle with parameters"
    );

    let monitors = wallpaper_manager.monitor_manager.get_monitors().await?;

    // Get monitor specific wallpapers
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
        // Determine which theme to use for this monitor
        let monitor_theme_id: &str = match monitor_specific_wallpapers.get(monitor_id) {
            Some(theme_id) => theme_id.as_ref(),
            None => continue,
        };

        info!(monitor_theme_id, monitor_id, "Determined theme for monitor");

        // Set the first valid theme as lock screen wallpaper
        if lock_screen_wallpaper.is_none() {
            lock_screen_wallpaper = Some(monitor_theme_id.to_string());
        }

        if let Err(e) = process_monitor_wallpaper(
            config,
            monitor_id,
            monitor_theme_id,
            &sun_position,
            wallpaper_manager,
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
        if let Some(theme_id) = lock_screen_wallpaper {
            if let Err(e) = set_lock_screen_wallpaper(config, &theme_id, &sun_position).await {
                warn!(
                    error = %e,
                    "Failed to set lock screen wallpaper, continuing with other operations"
                );
                // Continue execution even if lock screen wallpaper setting fails
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
            // Continue execution even if color mode setting fails
        }
    }

    Ok(())
}
