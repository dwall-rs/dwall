//! HTTP download service
//!
//! This module provides functionality for downloading files over HTTP.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::StatusCode;
use tauri::Runtime;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::error::DownloadError;
use super::task_manager::{DownloadProgress, DownloadTaskManager, ProgressEmitter};
use crate::error::DwallSettingsResult;

/// Service for downloading files over HTTP
pub(super) struct HttpDownloadService {
    client: reqwest::Client,
}

impl HttpDownloadService {
    /// Create a new downloader instance
    pub(super) fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| {
                error!(error = ?e, "Failed to create HTTP client");
                e
            })
            .unwrap();

        Self { client }
    }

    /// Build download URL for a theme
    pub(super) fn build_download_url(theme_id: &str) -> String {
        format!(
            "https://github.com/dwall-rs/dwall-assets/releases/download/themes/{}.zip",
            theme_id.replace(' ', ".")
        )
    }

    /// Download a file from a URL to a local path with progress tracking
    pub(super) async fn download_file<R: Runtime>(
        &self,
        url: &str,
        file_path: &Path,
        theme_id: &str,
        cancel_flag: Arc<AtomicBool>,
        progress_emitter: Option<&ProgressEmitter<'_, R>>,
        task_manager: &DownloadTaskManager,
    ) -> DwallSettingsResult<()> {
        debug!(theme_id = theme_id, url = %url, "Downloading file from URL");

        // Initiate download
        let response = self.client.get(url).send().await.map_err(|e| {
            let err = DownloadError::from(e);
            error!(
                theme_id = theme_id,
                url = %url,
                error = ?err,
                "Failed to establish connection for download"
            );
            err
        })?;

        if let Err(e) = response.error_for_status_ref() {
            if let StatusCode::NOT_FOUND = response.status() {
                error!(
                    theme_id = theme_id,
                    "The theme does not exist on the server"
                );
                return Err(DownloadError::NotFound(theme_id.to_string()).into());
            }

            error!(theme_id = theme_id, error = ?e, "Got an error response");
            return Err(e.into());
        }

        let response_header = response.headers();
        debug!(response_header = ?response_header, "Got response headers");

        let total_size = response.content_length().unwrap_or(0);
        let mut stream = response.bytes_stream();

        // Create file for writing
        let mut file = fs::File::create(file_path).await.map_err(|e| {
            error!(
                theme_id = theme_id,
                file_path = %file_path.display(),
                error = ?e,
                "Failed to create temp file"
            );
            e
        })?;

        // Download and write chunks
        let mut downloaded_bytes: u64 = 0;
        while let Some(chunk_result) = stream.next().await {
            // Check if download has been cancelled
            if task_manager.is_cancelled(&cancel_flag) {
                info!(theme_id = theme_id, "Download cancelled by user");
                return Err(DownloadError::Cancelled.into());
            }

            let chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(e) => {
                    error!(
                        theme_id = theme_id,
                        error = ?e,
                        "Failed to download chunk"
                    );
                    return Err(e.into());
                }
            };

            if let Err(e) = file.write_all(&chunk).await {
                error!(
                    theme_id = theme_id,
                    downloaded_bytes,
                    total_bytes = total_size,
                    error = ?e,
                    "Failed to write chunk to file"
                );
                return Err(e.into());
            };

            downloaded_bytes += chunk.len() as u64;

            // Emit progress if emitter is provided
            if let Some(emitter) = progress_emitter {
                emitter
                    .emit_progress(DownloadProgress {
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
        }

        info!(
            theme_id = theme_id,
            downloaded_bytes,
            total_bytes = total_size,
            "Successfully downloaded file"
        );

        Ok(())
    }
}
