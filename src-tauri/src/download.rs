use std::error::Error;
use std::time::Duration;
use std::{io::Cursor, path::PathBuf};

use dwall::config::Config;
use dwall::{APP_CONFIG_DIR, THEMES_DIR};
use futures_util::StreamExt;
use serde::Serialize;
use tauri::{Emitter, WebviewWindow};
use tokio::{fs, io::AsyncWriteExt};

use crate::error::DwallSettingsResult;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("{0}")]
    Connect(String),
    #[error("Unhandled Error: {0}")]
    Unknown(String),
}

impl From<reqwest::Error> for DownloadError {
    fn from(value: reqwest::Error) -> Self {
        let source = value
            .source()
            .map(|e| format!("{:?}", e))
            .unwrap_or("".to_string());

        if value.is_connect() {
            Self::Connect(source[43..source.len() - 1].to_string())
        } else {
            Self::Unknown(source)
        }
    }
}

#[derive(Serialize, Clone)]
struct ProgressPayload<'a> {
    id: &'a str,
    progress: u64,
    total: u64,
}

async fn download_theme<'a>(
    window: WebviewWindow,
    config: &Config<'a>,
    theme_id: &str,
) -> DwallSettingsResult<PathBuf> {
    trace!("Starting theme download process for theme: {}", theme_id);

    // Construct GitHub download URL
    let github_url = format!(
        "https://github.com/thep0y/dwall-assets/releases/download/themes/{}.zip",
        theme_id.replace(" ", ".")
    );
    debug!("Generated GitHub download URL: {}", github_url);

    let asset_url = config.github_asset_url(&github_url);
    info!("Attempting to download theme from URL: {}", asset_url);

    let client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(5))
        .build()?;

    // Perform download
    let response = match client.get(asset_url).send().await {
        Ok(resp) => {
            debug!("Successfully initiated download");
            resp
        }
        Err(e) => {
            error!("Failed to initiate download: {:?}", e);
            return Err(DownloadError::from(e).into());
        }
    };

    let total = response.content_length().unwrap_or(0);
    debug!("Total download size: {} bytes", total);

    let mut stream = response.bytes_stream();

    // Prepare theme directory
    let target_dir = APP_CONFIG_DIR.join("themes").join(theme_id);
    match fs::remove_dir_all(&target_dir).await {
        Ok(_) => trace!("Existing theme directory removed"),
        Err(e) => warn!("Failed to remove existing theme directory: {}", e),
    }

    match fs::create_dir(&target_dir).await {
        Ok(_) => trace!("Created new theme directory"),
        Err(e) => {
            error!("Failed to create theme directory: {}", e);
            return Err(e.into());
        }
    }

    // Prepare theme zip file
    let theme_zip_file = THEMES_DIR.join(format!("{}.zip", theme_id));
    let mut file = match fs::File::create(&theme_zip_file).await {
        Ok(f) => {
            debug!("Created theme zip file: {:?}", theme_zip_file);
            f
        }
        Err(e) => {
            error!("Failed to create theme zip file: {}", e);
            return Err(e.into());
        }
    };

    // Download and write chunks
    let mut downloaded_len: u64 = 0;
    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(chunk) => chunk,
            Err(e) => {
                error!("Error downloading chunk: {}", e);
                return Err(e.into());
            }
        };

        match file.write_all(&chunk).await {
            Ok(_) => {
                downloaded_len += chunk.len() as u64;
                trace!("Downloaded {} / {} bytes", downloaded_len, total);

                // Emit progress
                let _ = window.emit(
                    "download-theme",
                    ProgressPayload {
                        id: theme_id,
                        progress: downloaded_len,
                        total,
                    },
                );
            }
            Err(e) => {
                error!("Failed to write chunk to file: {}", e);
                return Err(e.into());
            }
        }
    }

    info!(
        "Successfully downloaded theme {} ({} bytes)",
        theme_id, downloaded_len
    );
    Ok(theme_zip_file)
}

#[tauri::command]
pub async fn download_theme_and_extract<'a>(
    window: WebviewWindow,
    config: Config<'a>,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    info!("Starting theme download and extraction for: {}", theme_id);

    // Download theme
    let file_path = match download_theme(window.clone(), &config, theme_id).await {
        Ok(path) => {
            debug!("Theme downloaded successfully to {:?}", path);
            path
        }
        Err(e) => {
            error!("Theme download failed: {}", e);
            return Err(e);
        }
    };

    // Read downloaded file
    let archive = match fs::read(&file_path).await {
        Ok(data) => {
            trace!("Read downloaded theme archive");
            data
        }
        Err(e) => {
            error!("Failed to read downloaded theme archive: {}", e);
            return Err(e.into());
        }
    };

    // Extract theme
    let target_dir = APP_CONFIG_DIR.join("themes").join(theme_id);
    match zip_extract::extract(Cursor::new(archive), &target_dir, true) {
        Ok(_) => {
            info!("Successfully extracted theme to {:?}", target_dir);
        }
        Err(e) => {
            error!("Theme extraction failed: {}", e);
            return Err(e.into());
        }
    };

    // Delete downloaded file
    match fs::remove_file(file_path).await {
        Ok(_) => {
            info!("Successfully delete theme archive");
            Ok(())
        }
        Err(e) => {
            error!("Failed to delete theme archive: {}", e);
            Err(e.into())
        }
    }
}
