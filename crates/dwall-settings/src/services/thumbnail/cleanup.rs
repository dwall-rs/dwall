//! Cache cleanup functionality
//!
//! This module provides functionality for cleaning up and maintaining the cache.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use dwall::DWALL_CACHE_DIR;
use tokio::fs;

use crate::error::DwallSettingsResult;

use super::storage::Storage;

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
        let thumbnails_dir = DWALL_CACHE_DIR.join("thumbnails");
        if !thumbnails_dir.exists() {
            return Ok(0);
        }

        let expired_files = Self::collect_expired_files(&thumbnails_dir).await?;
        let cleaned_bytes = Self::delete_files(&expired_files).await;
        Self::cleanup_empty_directories(&thumbnails_dir).await?;

        Ok(cleaned_bytes)
    }

    /// Enforce cache size limit by removing oldest files when cache is too large
    pub(crate) async fn enforce_cache_size_limit() -> DwallSettingsResult<u64> {
        let thumbnails_dir = DWALL_CACHE_DIR.join("thumbnails");
        if !thumbnails_dir.exists() {
            return Ok(0);
        }

        let current_size = Storage::get_directory_size(&thumbnails_dir).await?;

        if current_size <= MAX_CACHE_SIZE_BYTES {
            return Ok(0);
        }

        let cache_files = Self::collect_all_files(&thumbnails_dir).await?;
        let cleaned_bytes = Self::delete_oldest_files(&cache_files, current_size).await;
        Self::cleanup_empty_directories(&thumbnails_dir).await?;

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

        while let Ok(Some(entry)) = entries.next_entry().await {
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

        while let Ok(Some(file_entry)) = theme_entries.next_entry().await {
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
                    error!(file = %file_path.display(), "Failed to remove cache file");
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

        while let Ok(Some(entry)) = entries.next_entry().await {
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

        while let Ok(Some(file_entry)) = theme_entries.next_entry().await {
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

    /// Delete oldest files until target size is reached
    async fn delete_oldest_files(cache_files: &[CacheFileInfo], current_size: u64) -> u64 {
        let mut files: Vec<_> = cache_files.iter().collect();
        files.sort_by_key(|f| f.modified);

        let target_size = current_size - MAX_CACHE_SIZE_BYTES;
        let mut cleaned_bytes = 0;

        for file in files {
            if cleaned_bytes >= target_size {
                break;
            }

            if fs::remove_file(&file.path).await.is_ok() {
                cleaned_bytes += file.size;
                debug!(file = %file.path.display(), size = file.size, "Removed cache file");
            } else {
                error!(file = %file.path.display(), "Failed to remove cache file");
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

        while let Ok(Some(entry)) = entries.next_entry().await {
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
