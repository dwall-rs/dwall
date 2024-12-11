use std::error::Error;
use std::time::Duration;
use std::{io::Cursor, path::PathBuf};

use dwall::config::Config;
use dwall::THEMES_DIR;
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

/// Download progress tracking
#[derive(Serialize, Clone, Debug)]
struct DownloadProgress<'a> {
    theme_id: &'a str,
    downloaded_bytes: u64,
    total_bytes: u64,
}

/// Theme download and processing service
pub struct ThemeDownloader<'a> {
    client: reqwest::Client,
    window: &'a WebviewWindow,
}

impl<'a> ThemeDownloader<'a> {
    /// Create a new downloader instance
    pub fn new(window: &WebviewWindow) -> Self {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, window }
    }

    /// Build download URL for a theme
    fn build_download_url(theme_id: &str) -> String {
        format!(
            "https://github.com/thep0y/dwall-assets/releases/download/themes/{}.zip",
            theme_id.replace(' ', ".")
        )
    }

    /// Emit download progress notification
    fn emit_progress(&self, progress: DownloadProgress) -> Result<(), tauri::Error> {
        self.window.emit("download-theme", progress)
    }

    /// Download theme zip file
    #[instrument(skip(self, config), fields(theme_id = theme_id))]
    pub async fn download_theme<'a>(
        &self,
        config: &Config<'a>,
        theme_id: &str,
    ) -> DwallSettingsResult<PathBuf> {
        // Construct download URL
        let github_url = Self::build_download_url(theme_id);
        let asset_url = config.github_asset_url(&github_url);

        debug!("Downloading theme from URL: {}", asset_url);

        // Prepare target directories
        let target_dir = THEMES_DIR.join(theme_id);
        let theme_zip_file = THEMES_DIR.join(format!("{}.zip", theme_id));

        // Clean up existing directories
        self.prepare_theme_directory(&target_dir).await?;

        // Initiate download
        let response = self
            .client
            .get(&asset_url)
            .send()
            .await
            .map_err(|e| DownloadError::ConnectionError(e.to_string()))?;

        let total_size = response.content_length().unwrap_or(0);
        let mut stream = response.bytes_stream();

        // Create file for writing
        let mut file = fs::File::create(&theme_zip_file)
            .await
            .map_err(DownloadError::FileError)?;

        // Download and write chunks
        let mut downloaded_bytes: u64 = 0;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| DownloadError::RequestError(e.to_string()))?;

            file.write_all(&chunk)
                .await
                .map_err(DownloadError::FileError)?;

            downloaded_bytes += chunk.len() as u64;

            // Emit progress
            self.emit_progress(DownloadProgress {
                theme_id,
                downloaded_bytes,
                total_bytes: total_size,
            })
            .map_err(|e| DownloadError::ProgressError(e.to_string()))?;
        }

        info!(
            "Successfully downloaded theme {} ({} bytes)",
            theme_id, downloaded_bytes
        );
        Ok(theme_zip_file)
    }

    /// Prepare theme directory for download
    async fn prepare_theme_directory(&self, target_dir: &Path) -> Result<(), DownloadError> {
        // Remove existing directory if it exists
        if target_dir.exists() {
            fs::remove_dir_all(target_dir)
                .await
                .map_err(DownloadError::FileError)?;
            trace!("Removed existing theme directory");
        }

        // Create new directory
        fs::create_dir_all(target_dir)
            .await
            .map_err(DownloadError::FileError)?;

        trace!("Created new theme directory");
        Ok(())
    }

    /// Extract downloaded theme
    pub async fn extract_theme(&self, zip_path: &Path, theme_id: &str) -> DwallSettingsResult<()> {
        let target_dir = THEMES_DIR.join(theme_id);

        // Read downloaded file
        let archive = fs::read(zip_path).await.map_err(DownloadError::FileError)?;

        // Extract theme
        zip_extract::extract(std::io::Cursor::new(archive), &target_dir, true)
            .map_err(|e| DownloadError::ExtractionError(e.to_string()))?;

        info!("Successfully extracted theme to {:?}", target_dir);

        // Clean up zip file
        fs::remove_file(zip_path)
            .await
            .map_err(DownloadError::FileError)?;

        info!("Deleted theme archive");
        Ok(())
    }
}

#[tauri::command]
pub async fn download_theme_and_extract<'a>(
    window: WebviewWindow,
    config: Config<'a>,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    let downloader = ThemeDownloader::new(&window);

    // Download theme
    let zip_path = downloader.download_theme(&config, theme_id).await?;

    // Extract theme
    downloader.extract_theme(&zip_path, theme_id).await
}
