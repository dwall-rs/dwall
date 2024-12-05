use std::{path::Path, sync::Arc, time::Duration};

use time::{macros::offset, OffsetDateTime};
use tokio::{fs, time::sleep};

use crate::{
    color_mode::{determine_color_mode, set_color_mode},
    config::Config,
    position::{Position, PositionManager},
    solar::{SolarAngle, SunPosition},
    theme::{manager::WallpaperManager, ThemeError},
    DwallResult, THEMES_DIR,
};

/// Manages the lifecycle and processing of a specific theme
pub struct ThemeProcessor<'a> {
    /// Unique identifier for the current theme
    theme_id: String,
    /// Configuration settings for theme processing
    config: Arc<Config<'a>>,
    /// Manages geographic position tracking
    position_manager: PositionManager,
}

impl<'a> ThemeProcessor<'a> {
    /// Creates a new ThemeProcessor instance
    pub fn new(theme_id: &str, config: Arc<Config<'a>>) -> Self {
        debug!(theme_id = theme_id, "Initializing ThemeProcessor for theme");

        Self {
            theme_id: theme_id.to_string(),
            position_manager: PositionManager::new(config.coordinate_source().clone()),
            config,
        }
    }

    /// Starts a continuous loop to update theme based on current position
    pub async fn start_update_loop(&self) -> DwallResult<()> {
        info!("Starting theme update loop");

        loop {
            match self.position_manager.get_current_position().await {
                Ok(current_position) => match self.process_theme_cycle(&current_position).await {
                    Ok(_) => {
                        debug!("Theme cycle processed successfully");
                    }
                    Err(e) => {
                        error!(
                            error = %e,
                            theme_id = %self.theme_id,
                            "Failed to process theme cycle"
                        );
                        break;
                    }
                },
                Err(position_error) => {
                    error!(
                        error = %position_error,
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
            sleep(sleep_duration).await;
        }

        warn!("Theme update loop terminated");
        Ok(())
    }

    /// Process theme cycle for the current geographic position
    async fn process_theme_cycle(&self, position: &Position) -> DwallResult<()> {
        process_theme_cycle(
            &self.theme_id,
            self.config.auto_detect_color_mode(),
            self.config.image_format(),
            position,
        )
        .await
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
                error = %read_error,
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
                error = %parse_error,
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

/// Core theme processing function
async fn process_theme_cycle<'a, I: Into<&'a str>>(
    theme_id: &str,
    auto_detect_color_mode: bool,
    image_format: I,
    geographic_position: &Position,
) -> DwallResult<()> {
    let image_format: &'a str = image_format.into();
    let theme_directory = THEMES_DIR.join(theme_id);

    // Load solar angles for the theme
    let solar_angles = load_solar_angles(&theme_directory).await?;

    // Calculate current time with timezone offset
    let current_time = OffsetDateTime::now_utc().to_offset(offset!(+8));
    debug!(
        current_time = %current_time,
        timezone_offset = 8,
        "Calculating sun position"
    );

    // Compute sun position
    let sun_position = SunPosition::new(
        geographic_position.latitude,
        geographic_position.longitude,
        current_time,
        8,
    );

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
        .join(image_format)
        .join(format!("{}.jpg", closest_image_index + 1));

    info!(
        wallpaper_path = %wallpaper_path.display(),
        image_index = closest_image_index,
        "Selected wallpaper for current sun position"
    );

    // Update wallpapers
    WallpaperManager::set_lock_screen_image(&wallpaper_path)?;
    WallpaperManager::set_desktop_wallpaper(&wallpaper_path)?;

    // Optionally update system color mode
    if auto_detect_color_mode {
        let color_mode = determine_color_mode(altitude);
        info!(
            color_mode = ?color_mode,
            "Automatically updating system color mode"
        );
        set_color_mode(color_mode)?;
    }

    Ok(())
}
