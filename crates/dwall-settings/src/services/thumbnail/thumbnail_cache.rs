//! Thumbnail cache manager
//!
//! This module provides the main orchestrator for caching and managing thumbnail images.

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::OnceCell;

use crate::domain::theme::paths::ThemePathResolver;
use crate::error::DwallSettingsResult;
use crate::infrastructure::network::http_client::HttpClient;
use crate::services::thumbnail::cleanup::Cleanup;
use crate::services::thumbnail::store::clear_cache;

use super::storage::Storage;
use super::store::{CacheKey, CacheMetadata, get_or_create_cache_entry, initialize_cache};

/// Manages the thumbnail cache system
pub struct ThumbnailCache {
    http_client: HttpClient,
}

impl ThumbnailCache {
    pub fn new(client: HttpClient) -> Self {
        Self {
            http_client: client,
        }
    }

    /// Get or save a cached thumbnail
    pub(crate) async fn get_or_save_thumbnail(
        &self,
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

        let cache_key = CacheKey {
            theme_id: theme_id.to_string(),
            serial_number,
            url: url.to_string(),
        };

        let cell = get_or_create_cache_entry(cache_key.clone()).await;

        if let Some(path) = self.try_get_cached_file(&cell).await {
            return Ok(path);
        }

        self.trigger_probabilistic_cleanup().await;

        let image_path = self.download_and_cache_image(&cache_key, &cell).await?;

        Ok(image_path)
    }

    /// Try to get a cached file if it exists
    async fn try_get_cached_file(&self, cell: &Arc<OnceCell<CacheMetadata>>) -> Option<PathBuf> {
        if let Some(metadata) = cell.get() {
            debug!(path = %metadata.path.display(), "Found the cached image");

            if metadata.path.exists() {
                if let Err(e) = Storage::update_file_access_time(&metadata.path) {
                    warn!(path = %metadata.path.display(), error = %e, "Failed to update file access time");
                }
                return Some(metadata.path.clone());
            }

            warn!(
                path = %metadata.path.display(),
                "Cached file does not exist, will re-download"
            );
        }

        None
    }

    /// Trigger cache cleanup with a small probability
    async fn trigger_probabilistic_cleanup(&self) {
        if rand::random::<f32>() < 0.05 {
            tokio::spawn(async {
                if let Err(e) = Cleanup::cleanup_expired_cache().await {
                    error!(error = %e, "Failed to clean up expired cache");
                }

                if let Err(e) = Cleanup::enforce_cache_size_limit().await {
                    error!(error = %e, "Failed to enforce cache size limit");
                }
            });
        }
    }

    /// Download and cache an image
    async fn download_and_cache_image(
        &self,
        cache_key: &CacheKey,
        cell: &Arc<OnceCell<CacheMetadata>>,
    ) -> DwallSettingsResult<PathBuf> {
        let image_path = self.build_image_path(cache_key);

        initialize_cache().await?;
        Storage::ensure_directories(image_path.parent().unwrap()).await?;

        let path = if image_path.exists() {
            debug!(image_path = %image_path.display(), "Image already cached");
            image_path
        } else {
            debug!(url = cache_key.url, image_path = %image_path.display(), "Downloading image from URL");
            self.http_client
                .download_file(&cache_key.url, &image_path, 0, None, None)
                .await?;
            image_path
        };

        let metadata = CacheMetadata { path: path.clone() };
        let _ = cell.set(metadata);

        debug!(path = %path.display(), "Image successfully cached");
        Ok(path)
    }

    /// Build the image path for a cache key
    fn build_image_path(&self, cache_key: &CacheKey) -> PathBuf {
        let extension = get_url_extension(&cache_key.url).unwrap_or("jpg");

        ThemePathResolver::cached_image_path(
            &cache_key.theme_id,
            cache_key.serial_number,
            extension,
        )
    }
}

/// Extract file extension from URL
fn get_url_extension(url: &str) -> Option<&str> {
    let after_last_slash = url.rfind('/').map_or(url, |pos| &url[pos + 1..]);

    let clean_path = if let Some(query_pos) = after_last_slash.find('?') {
        &after_last_slash[..query_pos]
    } else if let Some(fragment_pos) = after_last_slash.find('#') {
        &after_last_slash[..fragment_pos]
    } else {
        after_last_slash
    };

    std::path::Path::new(clean_path)
        .extension()
        .and_then(|ext| ext.to_str())
}

/// Get or save a cached thumbnail
pub async fn get_or_save_cached_thumbnails(
    cache: &ThumbnailCache,
    theme_id: &str,
    serial_number: u8,
    url: &str,
) -> DwallSettingsResult<PathBuf> {
    cache
        .get_or_save_thumbnail(theme_id, serial_number, url)
        .await
}

/// Clear the thumbnail cache
pub async fn clear_thumbnail_cache() -> crate::error::DwallSettingsResult<u64> {
    info!("Manual cache cleanup requested");

    let expired_bytes = Cleanup::cleanup_expired_cache().await?;
    let size_limited_bytes = Cleanup::enforce_cache_size_limit().await?;

    let total_cleaned = expired_bytes + size_limited_bytes;
    info!(
        expired_bytes = expired_bytes,
        size_limited_bytes = size_limited_bytes,
        total_cleaned = total_cleaned,
        "Cache cleanup completed"
    );

    let cache_size = clear_cache().await;
    info!(
        cache_entries = cache_size,
        "Cleared in-memory thumbnail cache"
    );

    Ok(total_cleaned)
}
