//! Cache cleanup and maintenance.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use dwall::DWALL_CACHE_DIR;
use tokio::fs;

use crate::error::DwallSettingsResult;

use super::storage::{Storage, next_entry_or_stop};

// Cache configuration constants
const MAX_CACHE_SIZE_BYTES: u64 = 100 * 1024 * 1024; // 100MB
const CACHE_EXPIRY_DAYS: u64 = 30; // 30 days expiration

/// Information about a cached file
#[derive(Debug)]
struct CacheFileInfo {
    path: PathBuf,
    modified: SystemTime,
    size: u64,
}

/// Cache cleanup and maintenance
pub(crate) struct Cleanup;

impl Cleanup {
    /// Clean up expired cache files
    pub(crate) async fn cleanup_expired_cache() -> DwallSettingsResult<u64> {
        Self::cleanup_expired_in(&DWALL_CACHE_DIR.join("thumbnails")).await
    }

    /// Clean up expired cache files under `thumbnails_dir`.
    ///
    /// The directory is a parameter so the policy can be tested against a scratch
    /// cache instead of the user's.
    async fn cleanup_expired_in(thumbnails_dir: &Path) -> DwallSettingsResult<u64> {
        if !thumbnails_dir.exists() {
            return Ok(0);
        }

        let expired_files = Self::collect_expired_files(thumbnails_dir).await?;
        let cleaned_bytes = Self::delete_files(&expired_files).await;
        Self::cleanup_empty_directories(thumbnails_dir).await?;

        Ok(cleaned_bytes)
    }

    /// Enforce cache size limit by removing oldest files when cache is too large
    pub(crate) async fn enforce_cache_size_limit() -> DwallSettingsResult<u64> {
        Self::enforce_size_limit_in(&DWALL_CACHE_DIR.join("thumbnails"), MAX_CACHE_SIZE_BYTES).await
    }

    /// Enforce `max_bytes` under `thumbnails_dir`, removing the oldest files first.
    async fn enforce_size_limit_in(
        thumbnails_dir: &Path,
        max_bytes: u64,
    ) -> DwallSettingsResult<u64> {
        if !thumbnails_dir.exists() {
            return Ok(0);
        }

        let current_size = Storage::get_directory_size(thumbnails_dir).await?;

        if current_size <= max_bytes {
            return Ok(0);
        }

        let cache_files = Self::collect_all_files(thumbnails_dir).await?;
        let cleaned_bytes = Self::delete_oldest_files(&cache_files, current_size, max_bytes).await;
        Self::cleanup_empty_directories(thumbnails_dir).await?;

        Ok(cleaned_bytes)
    }

    /// Collect all expired files that should be removed
    async fn collect_expired_files(dir: &Path) -> DwallSettingsResult<Vec<PathBuf>> {
        let now = SystemTime::now();
        let expiry_duration = Duration::from_secs(CACHE_EXPIRY_DAYS * 24 * 60 * 60);
        let mut expired_files = Vec::new();

        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            error!(dir = %dir.display(), error = %e, "Failed to read directory");
            e
        })?;

        while let Some(entry) = next_entry_or_stop(&mut entries, dir).await {
            let path = entry.path();

            if path.is_dir() {
                Self::collect_expired_from_theme_dir(
                    &path,
                    &now,
                    expiry_duration,
                    &mut expired_files,
                )
                .await?;
            }
        }

        Ok(expired_files)
    }

    /// Collect expired files from a theme directory
    async fn collect_expired_from_theme_dir(
        dir: &Path,
        now: &SystemTime,
        expiry_duration: Duration,
        expired_files: &mut Vec<PathBuf>,
    ) -> DwallSettingsResult<()> {
        let mut theme_entries = match fs::read_dir(dir).await {
            Ok(entries) => entries,
            Err(e) => {
                error!(dir = %dir.display(), error = %e, "Failed to read theme directory");
                return Ok(());
            }
        };

        while let Some(file_entry) = next_entry_or_stop(&mut theme_entries, dir).await {
            let file_path = file_entry.path();

            if file_path.is_file()
                && let Ok(metadata) = fs::metadata(&file_path).await
                && let Ok(modified) = metadata.modified()
                && now
                    .duration_since(modified)
                    .is_ok_and(|age| age > expiry_duration)
            {
                expired_files.push(file_path);
            }
        }

        Ok(())
    }

    /// Delete a list of files and return total bytes cleaned
    async fn delete_files(files: &[PathBuf]) -> u64 {
        let mut cleaned_bytes = 0;

        for file_path in files {
            if let Ok(metadata) = fs::metadata(file_path).await {
                let file_size = metadata.len();
                if fs::remove_file(file_path).await.is_ok() {
                    cleaned_bytes += file_size;
                    debug!(file = %file_path.display(), size = file_size, "Removed cache file");
                } else {
                    // One file the cache could not delete is not an error: the walk
                    // continues and the file is simply retried on the next cleanup.
                    warn!(file = %file_path.display(), "Failed to remove cache file");
                }
            }
        }

        cleaned_bytes
    }

    /// Collect all files with their metadata
    async fn collect_all_files(dir: &Path) -> DwallSettingsResult<Vec<CacheFileInfo>> {
        let mut cache_files = Vec::new();
        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            error!(dir = %dir.display(), error = %e, "Failed to read directory");
            e
        })?;

        while let Some(entry) = next_entry_or_stop(&mut entries, dir).await {
            let path = entry.path();

            if path.is_dir() {
                Self::collect_files_from_theme_dir(&path, &mut cache_files).await?;
            }
        }

        Ok(cache_files)
    }

    /// Collect files from a theme directory
    async fn collect_files_from_theme_dir(
        dir: &Path,
        cache_files: &mut Vec<CacheFileInfo>,
    ) -> DwallSettingsResult<()> {
        let mut theme_entries = match fs::read_dir(dir).await {
            Ok(entries) => entries,
            Err(e) => {
                error!(dir = %dir.display(), error = %e, "Failed to read theme directory");
                return Ok(());
            }
        };

        while let Some(file_entry) = next_entry_or_stop(&mut theme_entries, dir).await {
            let file_path = file_entry.path();

            if file_path.is_file()
                && let Ok(metadata) = fs::metadata(&file_path).await
                && let Ok(modified) = metadata.modified()
            {
                cache_files.push(CacheFileInfo {
                    path: file_path,
                    modified,
                    size: metadata.len(),
                });
            }
        }

        Ok(())
    }

    /// Delete oldest files until the cache is under `max_bytes`
    async fn delete_oldest_files(
        cache_files: &[CacheFileInfo],
        current_size: u64,
        max_bytes: u64,
    ) -> u64 {
        let mut files: Vec<_> = cache_files.iter().collect();
        files.sort_by_key(|f| f.modified);

        let target_size = current_size - max_bytes;
        let mut cleaned_bytes = 0;

        for file in files {
            if cleaned_bytes >= target_size {
                break;
            }

            if fs::remove_file(&file.path).await.is_ok() {
                cleaned_bytes += file.size;
                debug!(file = %file.path.display(), size = file.size, "Removed cache file");
            } else {
                warn!(file = %file.path.display(), "Failed to remove cache file");
            }
        }

        cleaned_bytes
    }

    /// Clean empty directories recursively
    async fn cleanup_empty_directories(dir: &Path) -> DwallSettingsResult<()> {
        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            error!(dir = %dir.display(), error = %e, "Failed to read directory");
            e
        })?;

        while let Some(entry) = next_entry_or_stop(&mut entries, dir).await {
            let path = entry.path();

            if path.is_dir() {
                Box::pin(Self::cleanup_empty_directories(&path)).await?;

                if Storage::is_directory_empty(&path).await {
                    if fs::remove_dir(&path).await.is_ok() {
                        debug!(dir = %path.display(), "Removed empty directory");
                    } else {
                        error!(dir = %path.display(), "Failed to remove empty directory");
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing::ScratchDir;

    /// A scratch thumbnail cache.
    fn scratch(name: &str) -> ScratchDir {
        ScratchDir::new(&format!("cleanup_{name}"))
    }

    fn theme_dir(cache: &Path, theme: &str) -> PathBuf {
        let dir = cache.join(theme);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// Writes a file of `size` bytes with its modification time set `age_days` ago.
    fn write_aged_file(path: &Path, size: usize, age_days: u64) {
        std::fs::write(path, vec![b'x'; size]).unwrap();

        let file = std::fs::File::options().write(true).open(path).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(age_days * 24 * 60 * 60))
            .unwrap();
    }

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    #[test]
    fn a_missing_cache_directory_is_not_an_error() {
        let cache = scratch("missing");
        let _ = std::fs::remove_dir_all(&cache);

        assert_eq!(block_on(Cleanup::cleanup_expired_in(&cache)).unwrap(), 0);
        assert_eq!(
            block_on(Cleanup::enforce_size_limit_in(&cache, 1024)).unwrap(),
            0
        );
    }

    #[test]
    fn files_older_than_thirty_days_are_removed() {
        let cache = scratch("expiry");
        let themes = theme_dir(&cache, "Big Sur");
        write_aged_file(&themes.join("1.jpg"), 10, CACHE_EXPIRY_DAYS + 10);
        write_aged_file(&themes.join("2.jpg"), 10, 1);

        let cleaned = block_on(Cleanup::cleanup_expired_in(&cache)).unwrap();

        assert_eq!(cleaned, 10, "only the expired file's bytes are reported");
        assert!(!themes.join("1.jpg").exists(), "expired");
        assert!(themes.join("2.jpg").exists(), "still in use");
    }

    /// Reading a thumbnail keeps it alive: without that, a cache that is used every
    /// day still lost everything 30 days after the downloads.
    #[test]
    fn touching_a_file_keeps_it_out_of_the_expiry_set() {
        let cache = scratch("touched");
        let themes = theme_dir(&cache, "Big Sur");
        let thumbnail = themes.join("1.jpg");
        write_aged_file(&thumbnail, 10, CACHE_EXPIRY_DAYS + 10);

        // What the cache does on a hit.
        Storage::update_file_access_time(&thumbnail).unwrap();

        let cleaned = block_on(Cleanup::cleanup_expired_in(&cache)).unwrap();

        assert_eq!(cleaned, 0, "a file in use is not expired");
        assert!(thumbnail.exists());
    }

    #[test]
    fn empty_theme_directories_are_removed() {
        let cache = scratch("empty_dirs");
        let themes = theme_dir(&cache, "Big Sur");
        write_aged_file(&themes.join("1.jpg"), 10, CACHE_EXPIRY_DAYS + 10);
        // A directory that keeps a file, and one that is already empty.
        let kept = theme_dir(&cache, "Catalina");
        std::fs::write(kept.join("1.jpg"), vec![b'x'; 10]).unwrap();
        let already_empty = theme_dir(&cache, "Mojave");

        block_on(Cleanup::cleanup_expired_in(&cache)).unwrap();

        assert!(!themes.exists(), "the emptied theme directory goes too");
        assert!(kept.exists(), "a directory with files stays");
        assert!(
            !already_empty.exists(),
            "an empty directory is removed whether or not this run emptied it"
        );
    }

    #[test]
    fn a_cache_within_the_limit_is_left_alone() {
        let cache = scratch("under_limit");
        let themes = theme_dir(&cache, "Big Sur");
        write_aged_file(&themes.join("1.jpg"), 100, 1);

        assert_eq!(
            block_on(Cleanup::enforce_size_limit_in(&cache, 1024)).unwrap(),
            0
        );
        assert!(themes.join("1.jpg").exists());
    }

    /// The oldest files go first, and only until the cache is back under the limit.
    #[test]
    fn the_oldest_files_are_removed_until_the_cache_fits() {
        let cache = scratch("evict");
        let themes = theme_dir(&cache, "Big Sur");
        write_aged_file(&themes.join("old.jpg"), 100, 30);
        write_aged_file(&themes.join("middle.jpg"), 100, 10);
        write_aged_file(&themes.join("new.jpg"), 100, 1);

        // 300 bytes of files with a 150-byte limit: the two oldest have to go.
        let cleaned = block_on(Cleanup::enforce_size_limit_in(&cache, 150)).unwrap();

        assert_eq!(cleaned, 200);
        assert!(!themes.join("old.jpg").exists());
        assert!(!themes.join("middle.jpg").exists());
        assert!(
            themes.join("new.jpg").exists(),
            "the newest file is kept to reach the limit"
        );
    }

    #[test]
    fn the_cache_size_is_the_sum_of_its_theme_directories() {
        let cache = scratch("size");
        let first = theme_dir(&cache, "Big Sur");
        let second = theme_dir(&cache, "Catalina");
        std::fs::write(first.join("1.jpg"), vec![b'x'; 100]).unwrap();
        std::fs::write(second.join("1.jpg"), vec![b'x'; 50]).unwrap();

        assert_eq!(block_on(Storage::get_directory_size(&cache)).unwrap(), 150);
    }
}
