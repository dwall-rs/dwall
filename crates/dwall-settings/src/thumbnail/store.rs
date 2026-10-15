//! In-memory bookkeeping for thumbnail downloads.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use tokio::sync::Mutex;

use crate::error::DwallSettingsResult;
use crate::thumbnail::cleanup::Cleanup;

/// Cache key for identifying cached items
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub(crate) struct CacheKey {
    pub(crate) theme_id: String,
    pub(crate) serial_number: u8,
    pub(crate) url: String,
}

/// Per-key download locks.
///
/// Concurrent requests for the same thumbnail queue up behind one download
/// instead of fetching and writing the same file several times. The cached file
/// on disk stays the single source of truth, so a stale entry here is harmless.
type KeyLocks = Arc<Mutex<HashMap<CacheKey, Arc<Mutex<()>>>>>;

/// Global lock table for in-flight thumbnail downloads.
static THUMBNAIL_LOCKS: LazyLock<KeyLocks> = LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Returns the lock guarding downloads for `cache_key`.
pub(crate) async fn lock_for(cache_key: CacheKey) -> Arc<Mutex<()>> {
    let mut locks = THUMBNAIL_LOCKS.lock().await;

    locks
        .entry(cache_key)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

/// Clears the lock table, returning the number of tracked keys.
pub(crate) async fn clear_cache() -> usize {
    let mut locks = THUMBNAIL_LOCKS.lock().await;
    let size = locks.len();
    locks.clear();
    size
}

/// Acquires the download lock for `cache_key`.
///
/// The returned guard borrows the `Arc` returned alongside it, so callers must
/// keep both alive for the scope of the download.
pub(crate) async fn lock_for_download(
    cache_key: CacheKey,
) -> (Arc<Mutex<()>>, tokio::sync::OwnedMutexGuard<()>) {
    let lock = lock_for(cache_key).await;
    let guard = lock.clone().lock_owned().await;

    (lock, guard)
}

/// Initialize the cache system
pub(crate) async fn initialize_cache() -> DwallSettingsResult<()> {
    static CLEANUP_FLAG: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

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
