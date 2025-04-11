//! HTTP service for thumbnail downloads
//!
//! This module provides functionality for downloading thumbnail images over HTTP.

use std::fs;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;

use tokio::sync::Semaphore;
use tokio::time::sleep;

use super::error::{CacheError, CacheResult};

// 下载配置常量
const MAX_DOWNLOAD_RETRIES: u32 = 3; // 最大重试次数
const RETRY_DELAY_MS: u64 = 1000; // 重试延迟（毫秒）
const MAX_CONCURRENT_DOWNLOADS: usize = 5; // 最大并发下载数

// 下载信号量，控制并发下载数量
static DOWNLOAD_SEMAPHORE: LazyLock<Semaphore> =
    LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_DOWNLOADS));

/// Service for downloading images over HTTP
pub struct HttpService;

impl HttpService {
    /// Create HTTP client with appropriate timeouts
    pub async fn create_http_client() -> reqwest::Result<reqwest::Client> {
        debug!("Creating HTTP client");
        reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(120))
            .read_timeout(Duration::from_secs(120))
            .build()
    }

    /// Download an image from a URL to a local path
    pub async fn download_image(url: &str, image_path: &Path) -> CacheResult<u64> {
        debug!(url = url, "Downloading image");

        let temp_path = image_path.with_extension("temp");

        // 获取下载信号量许可，控制并发下载数量
        let _permit = DOWNLOAD_SEMAPHORE.acquire().await;
        debug!(url = url, "Acquired download semaphore permit");

        // 实现重试逻辑
        let mut retry_count = 0;
        let mut last_error = None;

        while retry_count < MAX_DOWNLOAD_RETRIES {
            if retry_count > 0 {
                warn!(
                    url = url,
                    retry = retry_count,
                    max_retries = MAX_DOWNLOAD_RETRIES,
                    "Retrying download after failure"
                );
                sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
            }

            let result = async {
                let client = Self::create_http_client().await.map_err(|e| {
                    error!(error = ?e, "Failed to create HTTP client");
                    CacheError::from(e)
                })?;

                trace!(url = url, "Sending HTTP GET request");
                let response = client.get(url).send().await.map_err(|e| {
                    error!(url = url, error = ?e, "Failed to get online image");
                    CacheError::from(e)
                })?;

                match response.error_for_status_ref() {
                    Ok(_) => {
                        trace!(url = url, "HTTP GET request succeeded");
                    }
                    Err(e) => {
                        error!(
                            url = url,
                            status_code = response.status().as_str(),
                            "HTTP GET request failed"
                        );
                        return Err(CacheError::from(e));
                    }
                }

                let content_length = response.content_length();
                trace!(url = url, length = ?content_length, "Received response. Reading bytes");

                let buffer = response.bytes().await.map_err(|e| {
                    error!(error = ?e, "Failed to read image bytes from response");
                    CacheError::from(e)
                })?;

                let file_size = buffer.len() as u64;

                trace!(
                    temp_path = %temp_path.display(),
                    "Writing image to temporary path"
                );
                fs::write(&temp_path, &buffer).map_err(|e| {
                    error!(
                        temp_path = %temp_path.display(),
                        error = ?e,
                        "Failed to write temporary image"
                    );
                    CacheError::from(e)
                })?;

                trace!(
                    from = %temp_path.display(),
                    to = %image_path.display(),
                    "Renaming temporary file to target path"
                );
                fs::rename(&temp_path, image_path).map_err(|e| {
                    error!(
                        from = %temp_path.display(),
                        to = %image_path.display(),
                        error = ?e,
                        "Failed to rename temporary file"
                    );
                    CacheError::from(e)
                })?;

                Ok(file_size)
            }
            .await;

            match result {
                Ok(size) => return Ok(size),
                Err(e) => {
                    last_error = Some(e);
                    retry_count += 1;
                }
            }
        }

        // 清理临时文件
        if temp_path.exists() {
            if let Err(e) = fs::remove_file(&temp_path) {
                error!(
                    temp_path = %temp_path.display(),
                    error = ?e,
                    "Failed to remove temporary file after error"
                );
            }
        }

        Err(last_error.unwrap_or_else(|| CacheError::DownloadFailed {
            url: url.to_string(),
            retries: MAX_DOWNLOAD_RETRIES,
        }))
    }

    /// Extract file extension from URL
    pub fn get_url_extension(url: &str) -> Option<&str> {
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
}
