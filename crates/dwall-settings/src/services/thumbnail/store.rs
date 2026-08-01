//! In-memory cache storage for thumbnails
//!
//! This module provides thread-safe in-memory storage for cached thumbnail metadata.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};

use tokio::sync::{Mutex, OnceCell};

use crate::error::DwallSettingsResult;
use crate::services::thumbnail::cleanup::Cleanup;

/// Cache key for identifying cached items
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub(crate) struct CacheKey {
    pub(crate) theme_id: String,
    pub(crate) serial_number: u8,
    pub(crate) url: String,
}

/// Metadata for a cached image
#[derive(Clone, Debug)]
pub(crate) struct CacheMetadata {
    pub(crate) path: PathBuf,
}

/// Type alias for the image cache
type ImageCache = Arc<Mutex<HashMap<CacheKey, Arc<OnceCell<CacheMetadata>>>>>;

/// Global thumbnail cache instance
static THUMBNAIL_CACHE: LazyLock<ImageCache> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Get or create a cache entry for the given key
pub(crate) async fn get_or_create_cache_entry(cache_key: CacheKey) -> Arc<OnceCell<CacheMetadata>> {
    let mut cache = THUMBNAIL_CACHE.lock().await;

    cache
        .entry(cache_key.clone())
        .or_insert_with(|| {
            debug!(
                theme_id = cache_key.theme_id,
                serial_number = cache_key.serial_number,
                url = cache_key.url,
                "Creating new OnceCell for cache key"
            );
            Arc::new(OnceCell::new())
        })
        .clone()
}

/// Clear the in-memory cache
pub(crate) async fn clear_cache() -> usize {
    let mut cache = THUMBNAIL_CACHE.lock().await;
    let size = cache.len();
    cache.clear();
    size
}

/// Initialize the cache system
pub(crate) async fn initialize_cache() -> DwallSettingsResult<()> {
    static CLEANUP_FLAG: OnceCell<()> = OnceCell::const_new();

    CLEANUP_FLAG
        .get_or_init(|| async {
            use dwall::DWALL_CACHE_DIR;

            let thumbnails_dir = DWALL_CACHE_DIR.join("thumbnails");
            if !thumbnails_dir.exists() {
                warn!(
                    path = %thumbnails_dir.display(),
                    "Thumbnails directory does not exist",
                );
            }

            // Perform cache cleanup during initialization
            match Cleanup::cleanup_expired_cache().await {
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
                        error = %e,
                        "Failed to clean up expired cache during initialization"
                    );
                }
            }
        })
        .await;

    Ok(())
}
