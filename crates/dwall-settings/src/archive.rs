//! Zip archive extraction.

use std::path::Path;

use tokio::fs;
use zip::read::root_dir_common_filter;

use crate::error::{DwallSettingsError, DwallSettingsResult};

/// Upper bound on the total uncompressed size declared by an archive.
const MAX_EXTRACTED_BYTES: u64 = 8 * 1024 * 1024 * 1024; // 8 GiB

/// Upper bound on the number of entries, so a zip bomb cannot create millions of
/// files either.
const MAX_ARCHIVE_ENTRIES: usize = 16_384;

/// Extract a zip archive to the target directory
///
/// Runs on a blocking thread (deflate + file writes) and streams from the archive
/// file instead of loading it into memory.
pub async fn extract_zip(zip_path: &Path, target_dir: &Path) -> DwallSettingsResult<()> {
    let zip_path = zip_path.to_path_buf();
    let target_dir = target_dir.to_path_buf();

    tauri::async_runtime::spawn_blocking(move || extract_zip_blocking(&zip_path, &target_dir))
        .await
        .map_err(|e| DwallSettingsError::Other(e.to_string()))?
}

fn extract_zip_blocking(zip_path: &Path, target_dir: &Path) -> DwallSettingsResult<()> {
    let file = std::fs::File::open(zip_path).map_err(|e| {
        error!(
            zip_path = %zip_path.display(),
            error = %e,
            "Failed to open zip archive"
        );
        e
    })?;

    let mut zip = zip::ZipArchive::new(file)?;

    if zip.len() > MAX_ARCHIVE_ENTRIES {
        return Err(DwallSettingsError::Other(format!(
            "Archive has too many entries ({} > {MAX_ARCHIVE_ENTRIES})",
            zip.len()
        )));
    }

    // Read the declared sizes from the central directory: this does not decompress
    // anything, but it catches a decompression bomb before it fills the disk.
    let mut declared_bytes = 0u64;
    for index in 0..zip.len() {
        let entry = zip.by_index(index)?;
        declared_bytes = declared_bytes.saturating_add(entry.size());

        if declared_bytes > MAX_EXTRACTED_BYTES {
            return Err(DwallSettingsError::Other(format!(
                "Archive would expand to more than {} GiB",
                MAX_EXTRACTED_BYTES / (1024 * 1024 * 1024)
            )));
        }
    }

    zip.extract_unwrapped_root_dir(target_dir, root_dir_common_filter)
        .map_err(|e| {
            error!(
                target_dir = %target_dir.display(),
                zip_path = %zip_path.display(),
                error = %e,
                "Failed to extract zip archive"
            );
            e
        })?;

    info!(
        target_dir = %target_dir.display(),
        "Successfully extracted archive"
    );

    Ok(())
}

/// Remove a file (used for cleanup after extraction)
pub async fn remove_file(path: &Path) -> DwallSettingsResult<()> {
    fs::remove_file(path).await.map_err(|e| {
        error!(
            path = %path.display(),
            error = %e,
            "Failed to remove file"
        );
        e.into()
    })
}
