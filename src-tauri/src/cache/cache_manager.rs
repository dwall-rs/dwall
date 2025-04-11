//! Cache manager for thumbnails
//!
//! This module provides the core functionality for managing the thumbnail cache.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use dwall::DWALL_CACHE_DIR;
use tokio::sync::{Mutex, OnceCell};

use crate::error::DwallSettingsResult;

use super::cleanup_service::CleanupService;
use super::error::{CacheError, CacheResult};
use super::fs_service::FsService;
use super::http_service::HttpService;

// Cache key type for better type safety
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct CacheKey {
    theme_id: String,
    serial_number: u8,
    url: String,
}

// 缓存项元数据
#[derive(Clone, Debug)]
struct CacheMetadata {
    path: PathBuf,
    // created_at: SystemTime,
    // size: u64,
}

type ImageCache = Arc<Mutex<HashMap<CacheKey, Arc<OnceCell<CacheMetadata>>>>>;

static THUMBNAIL_CACHE: LazyLock<ImageCache> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));
static CLEANUP_FLAG: OnceCell<()> = OnceCell::const_new();

/// Manages the thumbnail cache system
pub struct ThumbnailCache;

impl ThumbnailCache {
    /// Initialize the cache system
    async fn initialize_cache(thumbnails_dir: &Path) -> CacheResult<()> {
        CLEANUP_FLAG
            .get_or_init(|| async {
                if !thumbnails_dir.exists() {
                    warn!(
                        path = %thumbnails_dir.display(),
                        "Thumbnails directory does not exist",
                    );
                }

                // 初始化时执行一次缓存清理
                match CleanupService::cleanup_expired_cache().await {
                    Ok(cleaned_bytes) => {
                        if cleaned_bytes > 0 {
                            info!(
                                cleaned_bytes = cleaned_bytes,
                                "Cleaned up expired cache files during initialization"
                            );
                        }
                    }
                    Err(e) => {
                        error!(
                            error = ?e,
                            "Failed to clean up expired cache during initialization"
                        );
                    }
                }
            })
            .await;

        Ok(())
    }

    /// Get or save a cached thumbnail
    async fn get_or_save_thumbnail(
        theme_id: &str,
        serial_number: u8,
        url: &str,
    ) -> CacheResult<PathBuf> {
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

        let mut cache = THUMBNAIL_CACHE.lock().await;
        trace!("Acquired lock on thumbnail cache");

        let cell = cache
            .entry(cache_key.clone())
            .or_insert_with(|| {
                debug!(
                    theme_id = theme_id,
                    serial_number = serial_number,
                    url = url,
                    "Creating new OnceCell for cache key"
                );
                Arc::new(OnceCell::new())
            })
            .clone();
        drop(cache); // Release the lock early

        if let Some(metadata) = cell.get() {
            info!(
                path = %metadata.path.display(),
                "Found the cached image"
            );

            // 更新文件访问时间（如果文件存在）
            if metadata.path.exists() {
                if let Err(e) = FsService::update_file_access_time(&metadata.path) {
                    warn!(
                        path = %metadata.path.display(),
                        error = ?e,
                        "Failed to update file access time"
                    );
                }
                return Ok(metadata.path.clone());
            } else {
                // 缓存记录存在但文件不存在，需要重新下载
                warn!(
                    path = %metadata.path.display(),
                    "Cached file does not exist, will re-download"
                );
            }
        }

        let result = async {
            // 定期执行缓存清理（概率性触发，避免每次请求都执行）
            if rand::random::<f32>() < 0.05 {
                // 5%概率触发清理
                tokio::spawn(async {
                    if let Err(e) = CleanupService::cleanup_expired_cache().await {
                        error!(error = ?e, "Failed to clean up expired cache");
                    }

                    if let Err(e) = CleanupService::enforce_cache_size_limit().await {
                        error!(error = ?e, "Failed to enforce cache size limit");
                    }
                });
            }

            let thumbnails_dir = DWALL_CACHE_DIR.join("thumbnails");
            let theme_dir = thumbnails_dir.join(&cache_key.theme_id);

            debug!(
                thumbnails_dir = %thumbnails_dir.display(),
                theme_dir = %theme_dir.display(),
                "Ensuring directories exist"
            );
            Self::initialize_cache(&thumbnails_dir).await?;
            FsService::ensure_directories(&theme_dir).await?;

            let extension = HttpService::get_url_extension(&cache_key.url).unwrap_or("jpg");
            let image_path = theme_dir.join(format!("{}.{}", cache_key.serial_number, extension));

            if image_path.exists() {
                info!(
                    image_path = %image_path.display(),
                    "Image already cached"
                );

                // 获取文件大小
                let metadata = tokio::fs::metadata(&image_path).await.map_err(|e| {
                    error!(
                        path = %image_path.display(),
                        error = ?e,
                        "Failed to get file metadata"
                    );
                    CacheError::from(e)
                })?;

                return Ok((image_path, metadata.len()));
            }

            debug!(
                url = url,
                image_path = %image_path.display(),
                "Downloading image from URL"
            );
            let file_size = HttpService::download_image(&cache_key.url, &image_path).await?;

            info!(
                image_path = %image_path.display(),
                size = file_size,
                "Image successfully cached"
            );

            Ok((image_path, file_size))
        }
        .await;

        match result {
            Ok((path, size)) => {
                info!(
                    path = %path.display(),
                    size = size,
                    "Image successfully cached"
                );

                // 创建缓存元数据
                let metadata = CacheMetadata {
                    path: path.clone(),
                    // created_at: SystemTime::now(),
                    // size,
                };

                // 忽略初始化失败的错误（另一个任务可能已经初始化了它）
                let _ = cell.set(metadata);

                Ok(path)
            }
            Err(e) => Err(e),
        }
    }
}

/// Get or save a cached thumbnail (Tauri command)
#[tauri::command]
pub async fn get_or_save_cached_thumbnails(
    theme_id: &str,
    serial_number: u8,
    url: &str,
) -> DwallSettingsResult<PathBuf> {
    ThumbnailCache::get_or_save_thumbnail(theme_id, serial_number, url)
        .await
        .map_err(|e| e.into())
}

/// Clear the thumbnail cache (Tauri command)
#[tauri::command]
pub async fn clear_thumbnail_cache() -> DwallSettingsResult<u64> {
    info!("Manual cache cleanup requested");

    // 清理过期缓存
    let expired_bytes = CleanupService::cleanup_expired_cache().await?;

    // 强制执行大小限制
    let size_limited_bytes = CleanupService::enforce_cache_size_limit().await?;

    let total_cleaned = expired_bytes + size_limited_bytes;
    info!(
        expired_bytes = expired_bytes,
        size_limited_bytes = size_limited_bytes,
        total_cleaned = total_cleaned,
        "Cache cleanup completed"
    );

    // 清空内存缓存
    let mut cache = THUMBNAIL_CACHE.lock().await;
    let cache_size = cache.len();
    cache.clear();
    drop(cache);

    info!(
        cache_entries = cache_size,
        "Cleared in-memory thumbnail cache"
    );

    Ok(total_cleaned)
}
