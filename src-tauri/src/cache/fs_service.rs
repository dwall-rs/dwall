//! File system service for cache operations
//!
//! This module provides functionality for file system operations related to caching.

use std::fs::File;
use std::path::Path;

use tokio::fs;

use crate::fs::create_dir_if_missing;

use super::error::CacheResult;

/// Service for file system operations related to caching
pub struct FsService;

impl FsService {
    /// Ensure that the required directories exist
    pub async fn ensure_directories(theme_dir: &Path) -> CacheResult<()> {
        create_dir_if_missing(theme_dir).await.map_err(Into::into)
    }

    /// Update file access time to prevent premature expiration
    pub fn update_file_access_time(path: &Path) -> std::io::Result<()> {
        // Simply open and close the file to update access time
        File::open(path)?;
        Ok(())
    }

    /// Check if a directory is empty
    pub async fn is_directory_empty(dir: &Path) -> bool {
        match fs::read_dir(dir).await {
            Ok(mut entries) => matches!(entries.next_entry().await, Ok(None)),
            Err(_) => false,
        }
    }

    /// Get the size of a directory recursively
    pub async fn get_directory_size(dir: &Path) -> CacheResult<u64> {
        let mut total_size = 0;
        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            error!(
                dir = %dir.display(),
                error = %e,
                "Failed to read directory"
            );
            e
        })?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();

            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    total_size += metadata.len();
                }
            } else if path.is_dir() {
                total_size += Box::pin(Self::get_directory_size(&path)).await?;
            }
        }

        Ok(total_size)
    }
}
