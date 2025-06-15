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

/// Context for download stream processing
struct DownloadContext<'a, R: Runtime> {
    response: reqwest::Response,
    file: &'a mut fs::File,
    downloaded_bytes: &'a mut u64,
    total_size: u64,
    theme_id: &'a str,
    cancel_flag: Arc<AtomicBool>,
    progress_emitter: Option<&'a ProgressEmitter<'a, R>>,
    task_manager: &'a DownloadTaskManager,
}

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
                error!(error = %e, "Failed to create HTTP client");
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

        // Prepare file for writing and get initial downloaded bytes
        let (mut file, mut downloaded_bytes) = self.prepare_file(file_path, theme_id).await?;

        // Build request with resume support if needed
        let request = self.build_request(url, downloaded_bytes, theme_id);

        // Send request and validate response
        let response = self.send_request(request, url, theme_id).await?;

        // Process response and get total size
        let total_size = self
            .process_response(&response, downloaded_bytes, theme_id)
            .await?;

        // Download and process the file stream
        self.process_download_stream(DownloadContext {
            response,
            file: &mut file,
            downloaded_bytes: &mut downloaded_bytes,
            total_size,
            theme_id,
            cancel_flag,
            progress_emitter,
            task_manager,
        })
        .await?;

        info!(
            theme_id = theme_id,
            downloaded_bytes,
            total_bytes = total_size,
            "Successfully downloaded file"
        );

        Ok(())
    }

    /// Prepare file for writing and return file handle and current downloaded bytes
    async fn prepare_file(
        &self,
        file_path: &Path,
        theme_id: &str,
    ) -> DwallSettingsResult<(fs::File, u64)> {
        let mut downloaded_bytes: u64 = 0;
        let file = if file_path.exists() {
            // Get the size of existing file for resuming download
            let metadata = fs::metadata(file_path).await.map_err(|e| {
                error!(
                    theme_id = theme_id,
                    file_path = %file_path.display(),
                    error = %e,
                    "Failed to get metadata of existing temp file"
                );
                e
            })?;

            downloaded_bytes = metadata.len();
            debug!(
                theme_id = theme_id,
                downloaded_bytes, "Found existing temp file, resuming download"
            );

            // Open file in append mode
            fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(file_path)
                .await
                .map_err(|e| {
                    error!(
                        theme_id = theme_id,
                        file_path = %file_path.display(),
                        error = %e,
                        "Failed to open existing temp file"
                    );
                    e
                })?
        } else {
            // Create new file if it doesn't exist
            fs::File::create(file_path).await.map_err(|e| {
                error!(
                    theme_id = theme_id,
                    file_path = %file_path.display(),
                    error = %e,
                    "Failed to create temp file"
                );
                e
            })?
        };

        Ok((file, downloaded_bytes))
    }

    /// Build request with Range header if resuming
    fn build_request(
        &self,
        url: &str,
        downloaded_bytes: u64,
        theme_id: &str,
    ) -> reqwest::RequestBuilder {
        let mut request = self.client.get(url);
        if downloaded_bytes > 0 {
            let range = format!("bytes={}-", downloaded_bytes);
            request = request.header("Range", &range);
            debug!(theme_id = theme_id, range = range, "Setting Range header");
        }
        request
    }

    /// Send request and handle connection errors
    async fn send_request(
        &self,
        request: reqwest::RequestBuilder,
        url: &str,
        theme_id: &str,
    ) -> DwallSettingsResult<reqwest::Response> {
        let response = request.send().await.map_err(|e| {
            let err = DownloadError::from(e);
            error!(
                theme_id = theme_id,
                url = %url,
                error = %err,
                "Failed to establish connection for download"
            );
            err
        })?;

        let response_header = response.headers();
        debug!(response_header = ?response_header, "Got response headers");

        Ok(response)
    }

    /// Process response, validate status code and calculate total size
    async fn process_response(
        &self,
        response: &reqwest::Response,
        downloaded_bytes: u64,
        theme_id: &str,
    ) -> DwallSettingsResult<u64> {
        if let Err(e) = response.error_for_status_ref() {
            if let StatusCode::NOT_FOUND = response.status() {
                error!(
                    theme_id = theme_id,
                    url = %response.url(),
                    "The theme does not exist on the server"
                );
                return Err(DownloadError::NotFound(theme_id.to_string()).into());
            }

            error!(theme_id = theme_id, error = %e, "Got an error response");
            return Err(e.into());
        }

        let content_length = response.content_length().unwrap_or(0);
        let total_size = if downloaded_bytes > 0 && response.status() == StatusCode::PARTIAL_CONTENT
        {
            // For resumed downloads with 206 Partial Content response
            downloaded_bytes + content_length
        } else {
            // For new downloads or if server doesn't support range requests
            content_length
        };

        Ok(total_size)
    }

    /// Process download stream, handle cancellation, write chunks and report progress
    async fn process_download_stream<R: Runtime>(
        &self,
        context: DownloadContext<'_, R>,
    ) -> DwallSettingsResult<()> {
        let mut stream = context.response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            // Check if download has been cancelled
            if context.task_manager.is_cancelled(&context.cancel_flag) {
                info!(theme_id = context.theme_id, "Download cancelled by user");
                return Err(DownloadError::Cancelled.into());
            }

            let chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(e) => {
                    error!(
                        theme_id = context.theme_id,
                        error = %e,
                        "Failed to download chunk"
                    );
                    return Err(e.into());
                }
            };

            if let Err(e) = context.file.write_all(&chunk).await {
                error!(
                    theme_id = context.theme_id,
                    downloaded_bytes = *context.downloaded_bytes,
                    total_bytes = context.total_size,
                    error = %e,
                    "Failed to write chunk to file"
                );
                return Err(e.into());
            };

            *context.downloaded_bytes += chunk.len() as u64;

            // Emit progress if emitter is provided
            if let Some(emitter) = context.progress_emitter {
                self.emit_progress(
                    emitter,
                    context.theme_id,
                    *context.downloaded_bytes,
                    context.total_size,
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Emit download progress
    async fn emit_progress<R: Runtime>(
        &self,
        emitter: &ProgressEmitter<'_, R>,
        theme_id: &str,
        downloaded_bytes: u64,
        total_bytes: u64,
    ) -> DwallSettingsResult<()> {
        emitter
            .emit_progress(DownloadProgress {
                theme_id,
                downloaded_bytes,
                total_bytes,
            })
            .map_err(|e| {
                error!(
                    theme_id = theme_id,
                    downloaded_bytes,
                    total_bytes,
                    error = %e,
                    "Failed to emit download progress"
                );
                e
            })?;

        Ok(())
    }
}
