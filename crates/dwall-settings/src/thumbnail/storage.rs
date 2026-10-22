//! Thumbnail cache storage operations.

use std::fs::File;
use std::path::Path;

use tokio::fs;

use crate::error::DwallSettingsResult;
use crate::filesystem::create_dir_if_missing;

/// Reads the next directory entry, stopping the walk — and saying so — on an I/O error.
///
/// `while let Ok(Some(..))` ended the walk silently at the first error, so a cache that
/// could not be fully read looked like one that was fully clean.
pub(crate) async fn next_entry_or_stop(
    entries: &mut fs::ReadDir,
    dir: &Path,
) -> Option<fs::DirEntry> {
    match entries.next_entry().await {
        Ok(Some(entry)) => Some(entry),
        Ok(None) => None,
        Err(e) => {
            warn!(
                dir = %dir.display(),
                error = %e,
                "Stopping directory walk after a read error"
            );
            None
        }
    }
}

/// File system operations for cache storage
pub(crate) struct Storage;

impl Storage {
    /// Ensure that the required directories exist
    pub(crate) async fn ensure_directories(theme_dir: &Path) -> DwallSettingsResult<()> {
        create_dir_if_missing(theme_dir).await.map_err(Into::into)
    }

    /// Keep a cached file from expiring while it is still in use.
    ///
    /// Expiry is decided by the *modification* time (`Cleanup::collect_expired_files`),
    /// while merely opening a file only touches its access time — which Windows does
    /// not even maintain by default. A thumbnail used every day therefore still
    /// expired 30 days after it was downloaded, and the whole cache aged out the same
    /// way.
    pub(crate) fn update_file_access_time(path: &Path) -> std::io::Result<()> {
        let file = File::options().write(true).open(path)?;
        file.set_modified(std::time::SystemTime::now())?;

        Ok(())
    }

    /// Check if a directory is empty
    pub(crate) async fn is_directory_empty(dir: &Path) -> bool {
        match fs::read_dir(dir).await {
            Ok(mut entries) => matches!(entries.next_entry().await, Ok(None)),
            Err(_) => false,
        }
    }

    /// Get the size of a directory recursively
    pub(crate) async fn get_directory_size(dir: &Path) -> DwallSettingsResult<u64> {
        let mut total_size = 0;
        let mut entries = fs::read_dir(dir).await.map_err(|e| {
            error!(
                dir = %dir.display(),
                error = %e,
                "Failed to read directory"
            );
            e
        })?;

        while let Some(entry) = next_entry_or_stop(&mut entries, dir).await {
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
