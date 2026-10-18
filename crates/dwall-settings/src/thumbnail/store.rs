//! In-memory bookkeeping for thumbnail downloads.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use tokio::sync::Mutex;

use crate::error::DwallSettingsResult;
use crate::thumbnail::cleanup::Cleanup;

/// Cache key for identifying cached items
///
/// It identifies the *resource*, URL included, and
/// [`crate::theme::paths::ThemePathResolver::cached_image_path`] derives the file
/// name from all three fields: a key that mapped to a different file than the lock
/// did allowed two concurrent downloads to write one path.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn key(theme_id: &str, url: &str) -> CacheKey {
        CacheKey {
            theme_id: theme_id.to_string(),
            serial_number: 1,
            url: url.to_string(),
        }
    }

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    /// The lock table is process-wide, so each test uses keys of its own.
    #[test]
    fn the_same_key_returns_the_same_lock() {
        let first = block_on(lock_for(key("store-same", "https://a/1.jpg")));
        let second = block_on(lock_for(key("store-same", "https://a/1.jpg")));

        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn different_keys_get_different_locks() {
        let one = block_on(lock_for(key("store-diff", "https://a/1.jpg")));
        let two = block_on(lock_for(key("store-diff", "https://a/2.jpg")));

        assert!(!Arc::ptr_eq(&one, &two));
    }

    /// The two tests that touch the whole table at once take a lock of their own:
    /// clearing it releases the guard another test is holding.
    static TABLE_WIDE_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn table_wide_test_guard() -> std::sync::MutexGuard<'static, ()> {
        TABLE_WIDE_TEST
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Clearing the table is what makes the next download take a fresh lock instead
    /// of queueing behind a guard nobody holds any more.
    #[test]
    fn clearing_the_table_forgets_the_locks() {
        let _serialised = table_wide_test_guard();

        let before = block_on(lock_for(key("store-clear", "https://a/1.jpg")));

        let cleared = block_on(clear_cache());
        assert!(cleared >= 1, "the key that was just taken is counted");

        let after = block_on(lock_for(key("store-clear", "https://a/1.jpg")));
        assert!(!Arc::ptr_eq(&before, &after));
    }

    /// Holding the guard is what serialises two downloads of one thumbnail; the
    /// second `lock_for_download` has to wait for the first to drop.
    #[test]
    fn a_second_download_waits_for_the_first() {
        let _serialised = table_wide_test_guard();

        let (_, guard) = block_on(lock_for_download(key("store-single", "https://a/1.jpg")));

        let waited = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            runtime.block_on(async {
                let (_lock, _guard) =
                    lock_for_download(key("store-single", "https://a/1.jpg")).await;

                true
            })
        });

        // The waiting task cannot finish while the guard is held.
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(
            !waited.is_finished(),
            "the second download must queue behind the first"
        );

        drop(guard);
        assert!(waited.join().unwrap());
    }
}
