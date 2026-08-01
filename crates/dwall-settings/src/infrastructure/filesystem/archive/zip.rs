//! Pure zip extraction (technical implementation, no business knowledge)

use std::path::Path;

use tokio::fs;
use zip::read::root_dir_common_filter;

use crate::error::DwallSettingsResult;

/// Extract a zip archive to the target directory
pub async fn extract_zip(zip_path: &Path, target_dir: &Path) -> DwallSettingsResult<()> {
    let archive = fs::read(zip_path).await.map_err(|e| {
        error!(
            zip_path = %zip_path.display(),
            error = %e,
            "Failed to read zip archive"
        );
        e
    })?;

    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive))?;

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
