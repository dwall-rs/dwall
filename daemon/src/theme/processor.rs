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
pub struct ThemeProcessor<'a> {
    // /// Unique identifier for the current theme
    // theme_id: String,
    /// Configuration settings for theme processing
    config: &'a Config<'a>,
    /// Manages geographic position tracking
    position_manager: PositionManager,
}

impl<'a> ThemeProcessor<'a> {
    /// Creates a new ThemeProcessor instance
    pub fn new(theme_id: &str, config: &'a Config<'a>) -> Self {
        debug!(
            theme_id = theme_id,
            auto_detect_color_mode = ?config.auto_detect_color_mode(),
            image_format = ?config.image_format(),
            "Initializing ThemeProcessor for theme with configuration"
        );

        Self {
            //     theme_id: theme_id.to_string(),
            position_manager: PositionManager::new(config.coordinate_source().clone()),
            config,
        }
    }

    /// Starts a continuous loop to update theme based on current position
    pub async fn start_update_loop(&self) -> DwallResult<()> {
        info!("Starting theme update loop");

        let mut last_update_time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
        loop {
            let current_time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());
            debug!(
                last_update_time = %last_update_time,
                current_time = %current_time,
                "Beginning next theme update cycle"
            );

            match self.position_manager.get_current_position().await {
                Ok(current_position) => match self.process_theme_cycle(&current_position).await {
                    Ok(_) => {
                        debug!("Theme cycle processed successfully");
                    }
                    Err(e) => {
                        error!(
                            error = ?e,
                            "Failed to process theme cycle"
                        );
                        break;
                    }
                },
                Err(position_error) => {
                    error!(
                        error = ?position_error,
                        "Failed to retrieve current geographic position"
                    );
                    break;
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
        process_theme_cycle(self.config, position).await
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

/// Process theme cycle for a specific monitor
async fn process_monitor_wallpaper(
    config: &Config<'_>,
    monitor_id: &str,
    theme_id: &str,
    sun_position: &SunPosition,
) -> DwallResult<()> {
    let theme_directory = config.themes_directory().join(theme_id);

    // Load solar angles for the theme
    let solar_angles = load_solar_angles(&theme_directory).await?;

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
                    theme_id,
                    altitude, azimuth, "No suitable image found for current sun position"
                );
                ThemeError::ImageCountMismatch
            },
        )?;

    // Construct wallpaper path
    let wallpaper_path = theme_directory
        .join(std::convert::Into::<&str>::into(config.image_format()))
        .join(format!("{}.jpg", closest_image_index + 1));

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

    // Create WallpaperManager instance
    let wallpaper_manager = WallpaperManager::new()?;

    // Set wallpaper for the specific monitor
    wallpaper_manager.set_monitor_wallpaper(monitor_id, &wallpaper_path)?;

    Ok(())
}

/// Core theme processing function
async fn process_theme_cycle(
    config: &Config<'_>,
    geographic_position: &Position,
) -> DwallResult<()> {
    debug!(
        auto_detect_color_mode = config.auto_detect_color_mode(),
        image_format = ?config.image_format(),
        latitude = geographic_position.latitude,
        longitude = geographic_position.longitude,
        "Processing theme cycle with parameters"
    );

    let default_theme_id = config.default_theme_id()?;

    // Create WallpaperManager instance
    let wallpaper_manager = WallpaperManager::new()?;
    let monitors = wallpaper_manager.monitor_manager.get_monitors()?;

    // Get monitor specific wallpapers
    let monitor_specific_wallpapers = config.monitor_specific_wallpapers();

    let current_time = OffsetDateTime::now_utc();
    let sun_position = SunPosition::new(
        geographic_position.latitude,
        geographic_position.longitude,
        current_time,
        config.timezone_offset(),
    );

    // Process each monitor
    for monitor_id in monitors.keys() {
        // Determine which theme to use for this monitor
        let monitor_theme_id = monitor_specific_wallpapers
            .get(monitor_id)
            .map(|theme| theme.as_ref())
            .unwrap_or(default_theme_id);
        info!(monitor_theme_id, monitor_id, "Determined theme for monitor");

        if let Err(e) =
            process_monitor_wallpaper(config, monitor_id, monitor_theme_id, &sun_position).await
        {
            error!(
                error = ?e,
                monitor_id = monitor_id,
                theme_id = monitor_theme_id,
                "Failed to process wallpaper for monitor"
            );
            continue;
        }
    }

    // Update lock screen wallpaper if enabled
    if config.lock_screen_wallpaper_enabled() {
        let theme_directory = config.themes_directory().join(default_theme_id); // 锁屏壁纸使用默认主题
        let solar_angles = load_solar_angles(&theme_directory).await?;

        let closest_image_index = WallpaperManager::find_closest_image(
            &solar_angles,
            sun_position.altitude(),
            sun_position.azimuth(),
        )
        .ok_or_else(|| ThemeError::ImageCountMismatch)?;

        let wallpaper_path = theme_directory
            .join(std::convert::Into::<&str>::into(config.image_format()))
            .join(format!("{}.jpg", closest_image_index + 1));

        if wallpaper_path.exists() {
            WallpaperManager::set_lock_screen_image(&wallpaper_path)?;
        }
    }

    // Optionally update system color mode
    if config.auto_detect_color_mode() {
        let color_mode = determine_color_mode(sun_position.altitude());
        info!(
            color_mode = ?color_mode,
            "Automatically updating system color mode"
        );
        set_color_mode(color_mode)?;
    }

    Ok(())
}
