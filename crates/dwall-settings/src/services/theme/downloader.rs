//! Theme downloader service (use case orchestration)

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use tauri::Runtime;
use tokio::fs;
use tokio::time::sleep;

use crate::domain::config::MirrorUrlResolver;
use crate::domain::theme::download_url::ThemeDownloadUrlBuilder;
use crate::domain::theme::paths::ThemePathResolver;
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::infrastructure::filesystem::archive::zip;
use crate::infrastructure::filesystem::ops::create_dir_if_missing;
use crate::infrastructure::network::download::progress::{DownloadProgress, ProgressEmitter};
use crate::infrastructure::network::download::task_tracker::DownloadTaskManager;
use crate::infrastructure::network::http_client::HttpClient;

/// Orchestrates the theme download use case
pub struct ThemeDownloader {
    task_tracker: DownloadTaskManager,
    http_client: HttpClient,
}

impl ThemeDownloader {
    pub fn new(http_client: HttpClient) -> Self {
        Self {
            task_tracker: DownloadTaskManager::new(),
            http_client,
        }
    }

    /// Download and extract a theme
    pub async fn download_and_extract<R: Runtime>(
        &self,
        config: &dwall::Config,
        theme_id: &str,
        progress_emitter: Option<&ProgressEmitter<'_, R>>,
    ) -> DwallSettingsResult<PathBuf> {
        let cancel_flag = self.task_tracker.add_task(theme_id).await?;

        let github_url = ThemeDownloadUrlBuilder::build(theme_id);
        let asset_url = MirrorUrlResolver::resolve(config.network(), &github_url).await;

        let theme_dir = ThemePathResolver::theme_dir(config, theme_id);
        let zip_path = ThemePathResolver::zip_path(config, theme_id);
        let temp_path = ThemePathResolver::temp_zip_path(config, theme_id);

        self.prepare_directory(&theme_dir).await?;

        let download_result = self
            .download_with_retry(&asset_url, &temp_path, &cancel_flag, progress_emitter)
            .await;

        match download_result {
            Ok(_) => {
                self.finalize_download(&temp_path, &zip_path).await?;
                zip::extract_zip(&zip_path, &theme_dir).await?;
                zip::remove_file(&zip_path).await?;
                self.task_tracker.remove_task(theme_id).await;
                Ok(theme_dir)
            }
            Err(e) => {
                self.handle_download_error(&temp_path).await;
                self.task_tracker.remove_task(theme_id).await;
                Err(e)
            }
        }
    }

    /// Cancel a theme download
    pub async fn cancel(&self, theme_id: &str) {
        self.task_tracker.cancel_task(theme_id).await;
    }

    async fn prepare_directory(&self, dir: &std::path::Path) -> DwallSettingsResult<()> {
        if dir.exists() {
            fs::remove_dir_all(dir).await?;
        }
        create_dir_if_missing(dir).await?;
        Ok(())
    }

    async fn download_with_retry(
        &self,
        url: &str,
        temp_path: &std::path::Path,
        cancel_flag: &Arc<AtomicBool>,
        progress_emitter: Option<&ProgressEmitter<'_, impl Runtime>>,
    ) -> DwallSettingsResult<()> {
        const MAX_RETRIES: u32 = 3;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                warn!(attempt = attempt, "Retrying download");
                sleep(Duration::from_secs(1)).await;
            }

            let downloaded_bytes = if temp_path.exists() {
                fs::metadata(temp_path).await?.len()
            } else {
                0
            };

            match self
                .http_client
                .download_file(url, temp_path, downloaded_bytes, Some(cancel_flag), None)
                .await
            {
                Ok(total_size) => {
                    if let Some(emitter) = progress_emitter {
                        let _ = emitter.emit_progress(DownloadProgress {
                            task_id: "theme",
                            downloaded_bytes: total_size,
                            total_bytes: total_size,
                        });
                    }
                    return Ok(());
                }
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    warn!(error = %e, "Download failed, will retry");
                }
                Err(e) => return Err(e),
            }
        }

        Err(DwallSettingsError::Other(
            "Download failed after retries".to_string(),
        ))
    }

    async fn finalize_download(
        &self,
        temp_path: &std::path::Path,
        final_path: &std::path::Path,
    ) -> DwallSettingsResult<()> {
        fs::rename(temp_path, final_path).await?;
        Ok(())
    }

    async fn handle_download_error(&self, temp_path: &std::path::Path) {
        if let Ok(metadata) = fs::metadata(temp_path).await
            && metadata.len() == 0
        {
            let _ = fs::remove_file(temp_path).await;
        }
    }
}
