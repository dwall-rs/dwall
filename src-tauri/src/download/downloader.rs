//! Theme download coordination
//!
//! This module coordinates the theme download process using various components.

use std::path::PathBuf;

use dwall::config::Config;
use tauri::{Runtime, State, WebviewWindow};

use crate::error::DwallSettingsResult;

use super::extractor::ThemeExtractor;
use super::file_manager::ThemeFileManager;
use super::http_service::HttpDownloadService;
use super::task_manager::{DownloadTaskManager, ProgressEmitter};

/// Coordinates the theme download process
pub struct ThemeDownloader {
    download_service: HttpDownloadService,
    task_manager: DownloadTaskManager,
}

impl ThemeDownloader {
    /// Create a new theme downloader
    pub fn new() -> Self {
        Self {
            download_service: HttpDownloadService::new(),
            task_manager: DownloadTaskManager::new(),
        }
    }

    /// Download theme zip file
    async fn download_theme<R: Runtime>(
        &self,
        config: &Config,
        theme_id: &str,
        progress_emitter: Option<&ProgressEmitter<'_, R>>,
    ) -> DwallSettingsResult<PathBuf> {
        // Add download task and get the cancel flag
        let cancel_flag = self.task_manager.add_task(theme_id).await?;

        // Get file paths
        let (target_dir, temp_theme_zip_file, theme_zip_file) =
            ThemeFileManager::build_theme_paths(config, theme_id);

        // Prepare target directories
        ThemeFileManager::prepare_theme_directory(&target_dir).await?;

        // Construct download URL
        let github_url = HttpDownloadService::build_download_url(theme_id);
        let asset_url = config.github_asset_url(&github_url);

        // Download the file
        let download_result = self
            .download_service
            .download_file(
                &asset_url,
                &temp_theme_zip_file,
                theme_id,
                cancel_flag.clone(),
                progress_emitter,
                &self.task_manager,
            )
            .await;

        // Handle download result
        match download_result {
            Ok(_) => {
                // Finalize the download
                ThemeFileManager::finalize_download(&temp_theme_zip_file, &theme_zip_file).await?;
                // Remove download task from tracking
                self.task_manager.remove_task(theme_id).await;
                Ok(theme_zip_file)
            }
            Err(e) => {
                // Only clean up temporary file if it's a non-resumable error or empty file
                if let Ok(metadata) = tokio::fs::metadata(&temp_theme_zip_file).await {
                    if metadata.len() == 0 {
                        // Clean up empty temporary file
                        ThemeFileManager::cleanup_temp_file(&temp_theme_zip_file, false).await;
                    } else {
                        // Keep the partial download for future resume
                        debug!(
                            theme_id = theme_id,
                            "Keeping partial download for future resume"
                        );
                    }
                }
                // Remove download task from tracking
                self.task_manager.remove_task(theme_id).await;
                Err(e)
            }
        }
    }

    async fn cancel_theme_download(&self, theme_id: &str) {
        self.task_manager.cancel_task(theme_id).await;
    }
}

/// Download and extract a theme
#[tauri::command]
pub async fn download_theme_and_extract<R: Runtime>(
    window: WebviewWindow<R>,
    downloader: State<'_, ThemeDownloader>,
    config: Config,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    let progress_emitter = ProgressEmitter::new(&window);

    // Download theme
    let zip_path = downloader
        .download_theme(&config, theme_id, Some(&progress_emitter))
        .await?;

    // Extract theme
    ThemeExtractor::extract_theme(config.themes_directory(), &zip_path, theme_id).await
}

/// Cancel an ongoing theme download
#[tauri::command]
pub async fn cancel_theme_download(
    downloader: State<'_, ThemeDownloader>,
    theme_id: String,
) -> DwallSettingsResult<()> {
    downloader.cancel_theme_download(&theme_id).await;
    Ok(())
}
