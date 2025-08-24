//! Theme extraction functionality
//!
//! This module provides functionality for extracting downloaded theme archives.

use std::path::Path;

use tokio::fs;

use crate::error::DwallSettingsResult;

/// Theme extraction service
pub struct ThemeExtractor;

impl ThemeExtractor {
    /// Extract downloaded theme
    pub async fn extract_theme(
        themes_directory: &Path,
        zip_path: &Path,
        theme_id: &str,
    ) -> DwallSettingsResult<()> {
        let target_dir = themes_directory.join(theme_id);

        // Read downloaded file
        let archive = fs::read(zip_path).await.map_err(|e| {
            error!(
                theme_id = theme_id,
                zip_path = %zip_path.display(),
                error = %e,
                "Failed to read theme archive"
            );
            e
        })?;

        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(archive))?;

        // Extract theme
        zip.extract(&target_dir).map_err(|e| {
            error!(
                theme_id = theme_id,
                target_dir = %target_dir.display(),
                zip_path = %zip_path.display(),
                error = %e,
                "Failed to extract theme archive"
            );
            e
        })?;

        info!(
            theme_id = theme_id,
            target_dir = %target_dir.display(),
            "Successfully extracted theme"
        );

        // Clean up zip file
        fs::remove_file(zip_path).await.map_err(|e| {
            error!(
                theme_id = theme_id,
                zip_path = %zip_path.display(),
                error = %e,
                "Failed to delete theme archive"
            );
            e
        })?;

        info!(
            theme_id = theme_id,
            zip_path = %zip_path.display(),
            "Deleted theme archive"
        );
        Ok(())
    }
}
