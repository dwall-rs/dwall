use std::{fs, path::PathBuf, sync::LazyLock, time::Duration};

use crate::{error::DwallSettingsResult, fs::create_dir_if_missing};

pub static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::cache_dir().unwrap();

    let dir = config_dir.join("dwall");
    trace!("Initializing cache directory at: {}", dir.display());

    if !dir.exists() {
        if let Err(e) = fs::create_dir(&dir) {
            let error_message = format!("Failed to create cache dir: {}", e);
            error!("{}", error_message);
            panic!("{}", error_message);
        } else {
            info!("Cache directory created successfully at: {}", dir.display());
        }
    } else {
        debug!("Cache directory already exists at: {}", dir.display());
    }

    dir
});

#[tauri::command]
pub async fn get_or_save_cached_image(
    theme_id: &str,
    serial_number: u8,
    url: &str,
) -> DwallSettingsResult<PathBuf> {
    trace!(
        "Received request to cache image for theme: '{}' with serial number: {} and URL: {}",
        theme_id,
        serial_number,
        url
    );

    let theme_dir = CACHE_DIR.join(theme_id);
    create_dir_if_missing(&theme_dir).await?;

    let image_path = theme_dir.join(format!("{}.jpg", serial_number));
    trace!(
        "Checking if image already exists at: {}",
        image_path.display()
    );

    if image_path.exists() {
        info!("Image already cached at: {}", image_path.display());
        return Ok(image_path);
    }

    trace!(
        "Image not found in cache. Writing to path: {}",
        image_path.display()
    );

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(120))
        .read_timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| {
            error!("Failed to create HTTP client: {}", e);
            e
        })
        .unwrap();

    trace!("Sending HTTP GET request to: {}", url);
    let response = client.get(url).send().await.map_err(|e| {
        error!(url = url, error = ?e, "Failed to get online image");
        e
    })?;

    trace!("Received response from: {}. Reading bytes.", url);
    let buffer = response.bytes().await.map_err(|e| {
        error!("Failed to read image bytes from response: {}", e);
        e
    })?;

    trace!("Writing image to: {}", image_path.display());
    if let Err(e) = fs::write(&image_path, buffer) {
        error!("Failed to write image to {}: {}", image_path.display(), e);
        return Err(e.into());
    } else {
        info!("Image successfully cached at: {}", image_path.display());
    }

    Ok(image_path)
}
