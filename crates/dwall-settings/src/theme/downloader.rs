//! Theme download orchestration.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use tauri::Runtime;
use tokio::fs;
use tokio::time::sleep;

use crate::archive;
use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::filesystem::create_dir_if_missing;
use crate::network::HttpClient;
use crate::network::MirrorUrlResolver;
use crate::network::progress::{OwnedProgressEmitter, ProgressEmitter};
use crate::network::tasks::DownloadTaskManager;
use crate::theme::download_url::ThemeDownloadUrlBuilder;
use crate::theme::paths::ThemePathResolver;

/// Removes the download task from the tracker when dropped, so no early return can
/// leave the theme permanently marked as "already being processed".
struct DownloadTaskGuard<'a> {
    tracker: &'a DownloadTaskManager,
    theme_id: &'a str,
}

impl Drop for DownloadTaskGuard<'_> {
    fn drop(&mut self) {
        self.tracker.remove_task(self.theme_id);
    }
}

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
    ///
    /// The archive is extracted into a staging directory and only swapped in once
    /// it succeeded, so a failed download cannot destroy a working installation.
    pub async fn download_and_extract<R: Runtime>(
        &self,
        config: &dwall::Config,
        theme_id: &str,
        progress_emitter: Option<&ProgressEmitter<'_, R>>,
    ) -> DwallSettingsResult<PathBuf> {
        let cancel_flag = self.task_tracker.add_task(theme_id)?;
        let _task_guard = DownloadTaskGuard {
            tracker: &self.task_tracker,
            theme_id,
        };

        let github_url = ThemeDownloadUrlBuilder::build(theme_id);
        let asset_url = MirrorUrlResolver::resolve(config.network(), &github_url).await;

        let theme_dir = ThemePathResolver::theme_dir(config, theme_id);
        let staging_dir = ThemePathResolver::staging_dir(config, theme_id);
        let zip_path = ThemePathResolver::zip_path(config, theme_id);
        let temp_path = ThemePathResolver::temp_zip_path(config, theme_id);

        // Always start from a clean staging directory; the installed theme (if any)
        // is left untouched until the swap below.
        if staging_dir.exists() {
            fs::remove_dir_all(&staging_dir).await?;
        }
        create_dir_if_missing(&staging_dir).await?;

        let download_result = self
            .download_with_retry(
                &asset_url,
                &temp_path,
                &cancel_flag,
                progress_emitter,
                theme_id,
            )
            .await;

        if let Err(e) = download_result {
            cleanup_dir(&staging_dir).await;
            self.handle_download_error(&temp_path).await;
            return Err(e);
        }

        let extract_result = async {
            self.finalize_download(&temp_path, &zip_path).await?;
            archive::extract_zip(&zip_path, &staging_dir).await?;
            archive::remove_file(&zip_path).await
        }
        .await;

        if let Err(e) = extract_result {
            cleanup_dir(&staging_dir).await;
            return Err(e);
        }

        // Swap: drop the previous installation only now that a complete replacement
        // exists on disk.
        if theme_dir.exists()
            && let Err(e) = fs::remove_dir_all(&theme_dir).await
        {
            cleanup_dir(&staging_dir).await;
            return Err(e.into());
        }

        fs::rename(&staging_dir, &theme_dir).await.map_err(|e| {
            error!(
                theme_dir = %theme_dir.display(),
                error = %e,
                "Failed to move the extracted theme into place"
            );
            DwallSettingsError::from(e)
        })?;

        Ok(theme_dir)
    }

    /// Cancel a theme download
    pub async fn cancel(&self, theme_id: &str) {
        self.task_tracker.cancel_task(theme_id);
    }

    /// Applies a changed network configuration to the HTTP client.
    pub fn reconfigure(&self, network: Option<&dwall::config::Network>) -> DwallSettingsResult<()> {
        self.http_client.reconfigure(network)
    }

    async fn download_with_retry(
        &self,
        url: &str,
        temp_path: &std::path::Path,
        cancel_flag: &Arc<AtomicBool>,
        progress_emitter: Option<&ProgressEmitter<'_, impl Runtime>>,
        task_id: &str,
    ) -> DwallSettingsResult<()> {
        const MAX_RETRIES: u32 = 3;

        let progress_callback = progress_emitter
            .map(|emitter| OwnedProgressEmitter::new(emitter.window()).throttled_callback(task_id));

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
                .download_file(
                    url,
                    temp_path,
                    downloaded_bytes,
                    Some(cancel_flag),
                    progress_callback.clone(),
                )
                .await
            {
                Ok(_) => return Ok(()),
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
        // Keep a non-empty partial file: the next attempt resumes it with a range
        // request. Only a zero-byte leftover is useless.
        if let Ok(metadata) = fs::metadata(temp_path).await
            && metadata.len() == 0
        {
            let _ = fs::remove_file(temp_path).await;
        }
    }
}

/// Best-effort removal of a staging directory.
async fn cleanup_dir(dir: &std::path::Path) {
    if dir.exists()
        && let Err(e) = fs::remove_dir_all(dir).await
    {
        warn!(dir = %dir.display(), error = %e, "Failed to clean up the staging directory");
    }
}
