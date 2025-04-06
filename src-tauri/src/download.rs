use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use dwall::config::Config;
use futures_util::StreamExt;
use serde::Serialize;
use tauri::{Emitter, WebviewWindow};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::error::DwallSettingsResult;

/// Download task information
#[derive(Debug)]
struct DownloadTask {
    /// Flag to indicate if the download should be cancelled
    cancel: Arc<AtomicBool>,
}

static DOWNLOAD_TASKS: LazyLock<Arc<Mutex<HashMap<String, DownloadTask>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("{0}")]
    Connect(String),
    #[error("Download cancelled")]
    Cancelled,
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
    pub fn new(window: &'a WebviewWindow) -> Self {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| {
                error!(error = ?e, "Failed to create HTTP client");
                e
            })
            .unwrap();

        Self { client, window }
    }

    /// Build download URL for a theme
    fn build_download_url(theme_id: &str) -> String {
        format!(
            "https://github.com/dwall-rs/dwall-assets/releases/download/themes/{}.zip",
            theme_id.replace(' ', ".")
        )
    }

    /// Emit download progress notification
    fn emit_progress(&self, progress: DownloadProgress) -> Result<(), tauri::Error> {
        self.window.emit("download-theme", progress)
    }

    /// Download theme zip file
    pub async fn download_theme(
        &self,
        config: &Config,
        theme_id: &str,
    ) -> DwallSettingsResult<PathBuf> {
        // Check if theme is already being downloaded
        let mut tasks = DOWNLOAD_TASKS.lock().await;
        if tasks.contains_key(theme_id) {
            error!(theme_id = theme_id, "Theme is already being downloaded");
            return Err(
                DownloadError::Unknown("Theme is already being downloaded".to_string()).into(),
            );
        }
        // Mark theme as being downloaded with cancel flag
        let cancel_flag = Arc::new(AtomicBool::new(false));
        tasks.insert(
            theme_id.to_string(),
            DownloadTask {
                cancel: cancel_flag.clone(),
            },
        );
        drop(tasks);

        // Construct download URL
        let github_url = Self::build_download_url(theme_id);
        let asset_url = config.github_asset_url(&github_url);

        debug!(theme_id = theme_id, url = %asset_url, "Downloading theme from URL");

        // Prepare target directories
        let target_dir = config.themes_directory().join(theme_id);
        let temp_theme_zip_file = config
            .themes_directory()
            .join(format!("{}.zip.temp", theme_id));
        let theme_zip_file = config.themes_directory().join(format!("{}.zip", theme_id));

        // Clean up existing directories
        self.prepare_theme_directory(&target_dir).await?;

        // Initiate download
        let response = self.client.get(&asset_url).send().await.map_err(|e| {
            let err = DownloadError::from(e);
            error!(
                theme_id = theme_id,
                url = %asset_url,
                error = ?err,
                "Failed to establish connection for theme download"
            );
            err
        })?;

        if let Err(e) = response.error_for_status_ref() {
            error!(theme_id = theme_id, error = ?e, "Got an error response");
            return Err(e.into());
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut stream = response.bytes_stream();

        // Create file for writing
        let mut file = fs::File::create(&temp_theme_zip_file).await.map_err(|e| {
            error!(
                theme_id = theme_id,
                file_path = %temp_theme_zip_file.display(),
                error = ?e,
                "Failed to create temp theme zip file"
            );
            e
        })?;

        // Download and write chunks
        let mut downloaded_bytes: u64 = 0;
        while let Some(chunk_result) = stream.next().await {
            // Check if download has been cancelled
            if cancel_flag.load(Ordering::Relaxed) {
                info!(theme_id = theme_id, "Download cancelled by user");

                // Clean up temporary file
                if temp_theme_zip_file.exists() {
                    let _ = fs::remove_file(&temp_theme_zip_file).await;
                    debug!(file_path = %temp_theme_zip_file.display(), "Removed temporary download file");
                }

                // Remove download task from tracking
                let mut tasks = DOWNLOAD_TASKS.lock().await;
                tasks.remove(theme_id);

                return Err(DownloadError::Cancelled.into());
            }
            let chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(e) => {
                    error!(
                        theme_id = theme_id,
                        error = ?e,
                        "Failed to download theme chunk"
                    );
                    let mut tasks = DOWNLOAD_TASKS.lock().await;
                    tasks.remove(theme_id);
                    return Err(e.into());
                }
            };

            if let Err(e) = file.write_all(&chunk).await {
                error!(
                    theme_id = theme_id,
                    downloaded_bytes,
                    total_bytes = total_size,
                    error = ?e,
                    "Failed to write theme chunk to file"
                );
                let mut tasks = DOWNLOAD_TASKS.lock().await;
                tasks.remove(theme_id);
                return Err(e.into());
            };

            downloaded_bytes += chunk.len() as u64;

            // Emit progress
            self.emit_progress(DownloadProgress {
                theme_id,
                downloaded_bytes,
                total_bytes: total_size,
            })
            .map_err(|e| {
                error!(
                    theme_id = theme_id,
                    downloaded_bytes,
                    total_bytes = total_size,
                    error = ?e,
                    "Failed to emit download progress"
                );
                e
            })?;
        }

        info!(
            theme_id = theme_id,
            downloaded_bytes,
            total_bytes = total_size,
            "Successfully downloaded theme"
        );

        // Remove download task from tracking
        let mut tasks = DOWNLOAD_TASKS.lock().await;
        tasks.remove(theme_id);

        fs::rename(temp_theme_zip_file, &theme_zip_file).await?;

        Ok(theme_zip_file)
    }

    /// Prepare theme directory for download
    async fn prepare_theme_directory(&self, target_dir: &Path) -> DwallSettingsResult<()> {
        // Remove existing directory if it exists
        if target_dir.exists() {
            fs::remove_dir_all(target_dir).await.map_err(|e| {
                error!(
                    dir_path = %target_dir.display(),
                    error = ?e,
                    "Failed to remove existing theme directory"
                );
                e
            })?;
            trace!("Removed existing theme directory");
        }

        // Create new directory
        fs::create_dir_all(target_dir).await.map_err(|e| {
            error!(
                dir_path = %target_dir.display(),
                error = ?e,
                "Failed to create theme directory"
            );
            e
        })?;

        trace!(dir_path = %target_dir.display(), "Created new theme directory");
        Ok(())
    }

    /// Extract downloaded theme
    pub async fn extract_theme(
        &self,
        themes_directory: &Path,
        zip_path: &Path,
        theme_id: &str,
    ) -> DwallSettingsResult<()> {
        let target_dir = themes_directory.join(theme_id);

        // Read downloaded file
        let archive = fs::read(zip_path).await.map_err(|e| {
            error!(
                theme_id = theme_id,
                zip_path = %zip_path.display(),
                error = ?e,
                "Failed to read theme archive"
            );
            e
        })?;

        // Extract theme
        zip_extract::extract(std::io::Cursor::new(archive), &target_dir, true).map_err(|e| {
            error!(
                theme_id = theme_id,
                target_dir = %target_dir.display(),
                error = ?e,
                "Failed to extract theme archive"
            );
            e
        })?;

        info!(
            theme_id = theme_id,
            target_dir = %target_dir.display(),
            "Successfully extracted theme"
        );

        // Clean up zip file
        fs::remove_file(zip_path).await.map_err(|e| {
            error!(
                theme_id = theme_id,
                zip_path = %zip_path.display(),
                error = ?e,
                "Failed to delete theme archive"
            );
            e
        })?;

        info!(
            theme_id = theme_id,
            zip_path = %zip_path.display(),
            "Deleted theme archive"
        );
        Ok(())
    }
}

#[tauri::command]
pub async fn download_theme_and_extract(
    window: WebviewWindow,
    config: Config,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    let downloader = ThemeDownloader::new(&window);

    // Download theme
    let zip_path = downloader.download_theme(&config, theme_id).await?;

    // Extract theme
    downloader
        .extract_theme(config.themes_directory(), &zip_path, theme_id)
        .await
}

/// Cancel an ongoing theme download
#[tauri::command]
pub async fn cancel_theme_download(theme_id: String) -> DwallSettingsResult<()> {
    let tasks = DOWNLOAD_TASKS.lock().await;

    if let Some(task) = tasks.get(&theme_id) {
        // Set the cancel flag to true
        task.cancel.store(true, Ordering::Relaxed);
        info!(
            theme_id = theme_id,
            "Requested cancellation of theme download"
        );
        Ok(())
    } else {
        // Theme is not being downloaded
        warn!(
            theme_id = theme_id,
            "Attempted to cancel download for theme that is not being downloaded"
        );
        Ok(())
    }
}
