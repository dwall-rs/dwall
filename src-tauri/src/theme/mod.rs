use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};

use time::{macros::offset, OffsetDateTime};
use tokio::{
    sync::{mpsc, Mutex},
    time::sleep,
};

use crate::{
    color_mode::{determine_color_mode, set_color_mode},
    config::{write_config_file, Config},
    error::{DwallError, DwallResult},
    geo::get_geo_position,
    lazy::APP_CONFIG_DIR,
    solar::{SolarAngle, SunPosition},
};

use self::manager::WallpaperManager;

pub use self::validator::ThemeValidator;

mod manager;
mod validator;

/// Directory for storing theme configurations
pub static THEMES_DIR: LazyLock<PathBuf> = LazyLock::new(|| APP_CONFIG_DIR.join("themes"));

/// Comprehensive error handling for theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme does not exist")]
    NotExists,
    #[error("Missing solar configuration file")]
    MissingSolarConfigFile,
    #[error("Image count does not match solar configuration")]
    ImageCountMismatch,
    #[error("Channel communication error: {0}")]
    ChannelSend(String),
}

/// Type alias for the task closure sender
pub type CloseTaskSender = Arc<Mutex<Option<mpsc::Sender<()>>>>;

/// Closes the current theme task if one is running
#[tauri::command]
pub async fn close_last_theme_task(sender: tauri::State<'_, CloseTaskSender>) -> DwallResult<()> {
    if let Some(tx) = sender.lock().await.take() {
        trace!("Sending close signal to theme task");
        tx.send(()).await.map_err(|e| {
            error!("Failed to send close signal: {}", e);
            ThemeError::ChannelSend(e.to_string())
        })?;
    }

    Ok(())
}

/// Applies a theme and starts a background task for periodic wallpaper updates
#[tauri::command]
pub async fn apply_theme(
    sender: tauri::State<'_, CloseTaskSender>,
    config: Config,
) -> DwallResult<()> {
    let theme_id = config.theme_id();
    let config = Arc::new(config);

    trace!("Applying theme: {:?}", theme_id);

    if let Some(theme_id) = theme_id {
        ThemeValidator::validate_theme(&theme_id).await?;

        let (tx, mut rx) = mpsc::channel::<()>(1);

        tauri::async_runtime::spawn(async move {
            // Process initial theme cycle
            if let Err(e) = process_theme_cycle_and_save_config(&theme_id, config.clone()).await {
                error!("Initial theme cycle error: {}", e);
                return Err(e);
            }

            loop {
                let config = Arc::clone(&config);
                tokio::select! {
                    _ = sleep(Duration::from_secs(config.interval().into())) => {
                       if let Err(e) = process_theme_cycle_and_save_config(&theme_id, config.clone()).await {
                            error!("Periodic theme cycle error: {}", e);
                            break;
                       }
                    },
                    _ = rx.recv() => {
                        info!("Received exit signal, terminating theme task");
                        break;
                    }
                }
            }
            Ok::<(), DwallError>(())
        });

        let sender = sender.clone();
        let mut sender = sender.lock().await;
        *sender = Some(tx);
    }

    Ok(())
}

/// Process theme cycle and save configuration
async fn process_theme_cycle_and_save_config(
    theme_id: &str,
    config: Arc<Config>,
) -> DwallResult<()> {
    match process_theme_cycle(theme_id, config.image_format()) {
        Ok(_) => write_config_file(config).await.map_err(|e| {
            error!("Failed to save configuration: {}", e);
            e
        }),
        Err(e) => Err(e),
    }
}

/// Load solar configuration for a given theme
fn load_solar_angles(theme_dir: &Path) -> DwallResult<Vec<SolarAngle>> {
    let solar_config_path = theme_dir.join("solar.json");

    if !solar_config_path.exists() {
        error!(
            "Solar configuration file missing: {}",
            solar_config_path.display()
        );
        return Err(ThemeError::MissingSolarConfigFile.into());
    }

    let solar_config_content = fs::read_to_string(&solar_config_path).map_err(|e| {
        error!("Failed to read solar configuration: {}", e);
        e
    })?;

    let solar_angles: Vec<SolarAngle> =
        serde_json::from_str(&solar_config_content).map_err(|e| {
            error!("Failed to parse solar configuration JSON: {}", e);
            e
        })?;

    debug!(
        "Loaded {} solar angles from configuration",
        solar_angles.len()
    );

    Ok(solar_angles)
}

/// Core theme processing function
fn process_theme_cycle<'a, I: Into<&'a str>>(theme_id: &str, image_format: I) -> DwallResult<()> {
    let image_format: &'a str = image_format.into();
    let geographic_position = get_geo_position()?;

    let theme_dir = THEMES_DIR.join(theme_id);
    let solar_angles = load_solar_angles(&theme_dir)?;

    let current_time = OffsetDateTime::now_utc().to_offset(offset!(+8));
    debug!("Current local time: {}", current_time);

    let sun_position = SunPosition::new(
        geographic_position.latitude,
        geographic_position.longitude,
        current_time,
        8,
    );

    let altitude = sun_position.altitude();
    let azimuth = sun_position.azimuth();
    info!(
        "Solar angles - Elevation: {:.1}°, Azimuth: {:.1}°",
        altitude, azimuth
    );

    let closest_image_index = WallpaperManager::find_closest_image(
        &solar_angles,
        altitude,
        azimuth,
        geographic_position.latitude,
        current_time,
    )
    .ok_or_else(|| {
        error!("No suitable image found");
        ThemeError::ImageCountMismatch
    })?;

    let wallpaper_path = theme_dir
        .join(image_format)
        .join(format!("{}.jpg", closest_image_index + 1));

    // Update wallpapers and system color mode
    WallpaperManager::set_lock_screen_image(&wallpaper_path)?;
    WallpaperManager::set_desktop_wallpaper(&wallpaper_path)?;

    let color_mode = determine_color_mode(altitude);
    info!("Determined color mode: {:?}", color_mode);
    set_color_mode(color_mode)?;

    Ok(())
}
