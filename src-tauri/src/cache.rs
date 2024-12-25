use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use dwall::DWALL_CACHE_DIR;

use crate::{error::DwallSettingsResult, fs::create_dir_if_missing};

fn get_url_extension(url: &str) -> Option<&str> {
    let after_last_slash = url.rfind('/').map_or(url, |pos| &url[pos + 1..]);

    let clean_path = if let Some(query_pos) = after_last_slash.find('?') {
        &after_last_slash[..query_pos]
    } else if let Some(fragment_pos) = after_last_slash.find('#') {
        &after_last_slash[..fragment_pos]
    } else {
        after_last_slash
    };

    Path::new(clean_path)
        .extension()
        .and_then(|ext| ext.to_str())
}

#[tauri::command]
pub async fn get_or_save_cached_thumbnails(
    theme_id: &str,
    serial_number: u8,
    url: &str,
) -> DwallSettingsResult<PathBuf> {
    trace!(
        theme_id = theme_id,
        serial_number = serial_number,
        url = url,
        "Received request to cache image"
    );

    let theme_dir = DWALL_CACHE_DIR.join("thumbnails").join(theme_id);
    create_dir_if_missing(&theme_dir).await?;

    let extension = get_url_extension(url).unwrap_or("jpg");
    debug!(extension = extension, "Determined image extension");

    let image_path = theme_dir.join(format!("{}.{}", serial_number, extension));
    trace!(
        image_path = image_path.display().to_string(),
        "Checking if image already exists"
    );

    if image_path.exists() {
        info!(
            image_path = image_path.display().to_string(),
            "Image already cached"
        );
        return Ok(image_path);
    }

    trace!(
        image_path = image_path.display().to_string(),
        "Image not found in cache. Writing to path"
    );

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(120))
        .read_timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| {
            error!(error = ?e, "Failed to create HTTP client");
            e
        })?;

    trace!(url = url, "Sending HTTP GET request");
    let response = client.get(url).send().await.map_err(|e| {
        error!(url = url, error = ?e, "Failed to get online image");
        e
    })?;

    trace!(url = url, "Received response. Reading bytes");
    let buffer = response.bytes().await.map_err(|e| {
        error!(error = ?e, "Failed to read image bytes from response");
        e
    })?;

    trace!(
        image_path = image_path.display().to_string(),
        "Writing image to path"
    );
    if let Err(e) = fs::write(&image_path, buffer) {
        error!(
            image_path = image_path.display().to_string(),
            error = ?e,
            "Failed to write image"
        );
        return Err(e.into());
    } else {
        info!(
            image_path = image_path.display().to_string(),
            "Image successfully cached"
        );
    }

    Ok(image_path)
}
