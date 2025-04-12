//! Theme file management
//!
//! This module provides functionality for managing theme files and directories.

use std::path::{Path, PathBuf};

use dwall::config::Config;
use tokio::fs;

use crate::error::DwallSettingsResult;

/// Handles file system operations for theme management
pub(super) struct ThemeFileManager;

impl ThemeFileManager {
    /// Build paths for theme files
    pub(super) fn build_theme_paths(
        config: &Config,
        theme_id: &str,
    ) -> (PathBuf, PathBuf, PathBuf) {
        let target_dir = config.themes_directory().join(theme_id);
        let temp_theme_zip_file = config
            .themes_directory()
            .join(format!("{}.zip.temp", theme_id));
        let theme_zip_file = config.themes_directory().join(format!("{}.zip", theme_id));

        (target_dir, temp_theme_zip_file, theme_zip_file)
    }

    /// Prepare theme directory for download
    pub(super) async fn prepare_theme_directory(target_dir: &Path) -> DwallSettingsResult<()> {
        // Remove existing directory if it exists
        if target_dir.exists() {
            fs::remove_dir_all(target_dir).await.map_err(|e| {
                error!(
                    dir_path = %target_dir.display(),
                    error = ?e,
                    "Failed to remove existing theme directory"
                );
                e
            })?;
            trace!("Removed existing theme directory");
        }

        // Create new directory
        fs::create_dir_all(target_dir).await.map_err(|e| {
            error!(
                dir_path = %target_dir.display(),
                error = ?e,
                "Failed to create theme directory"
            );
            e
        })?;

        trace!(dir_path = %target_dir.display(), "Created new theme directory");
        Ok(())
    }

    /// Clean up temporary file if download is cancelled or failed
    ///
    /// If force_cleanup is false, the file will only be removed if it's empty
    pub async fn cleanup_temp_file(temp_file_path: &Path, force_cleanup: bool) {
        if temp_file_path.exists() {
            // Check if we should keep the file for resuming download
            if !force_cleanup {
                if let Ok(metadata) = fs::metadata(temp_file_path).await {
                    if metadata.len() > 0 {
                        debug!(file_path = %temp_file_path.display(), size = metadata.len(), "Keeping temporary file for resume");
                        return;
                    }
                }
            }

            // Remove the file
            if let Err(e) = fs::remove_file(temp_file_path).await {
                error!(
                    file_path = %temp_file_path.display(),
                    error = ?e,
                    "Failed to remove temporary file"
                );
            } else {
                debug!(file_path = %temp_file_path.display(), "Removed temporary download file");
            }
        }
    }

    /// Rename temporary file to final file
    pub(super) async fn finalize_download(
        temp_file_path: &Path,
        final_file_path: &Path,
    ) -> DwallSettingsResult<()> {
        fs::rename(temp_file_path, final_file_path)
            .await
            .map_err(|e| {
                error!(
                    from = %temp_file_path.display(),
                    to = %final_file_path.display(),
                    error = ?e,
                    "Failed to rename temporary file to final file"
                );
                e.into()
            })
    }
}
