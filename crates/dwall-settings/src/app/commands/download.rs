//! Download commands

use tauri::State;

use crate::error::DwallSettingsResult;
use crate::infrastructure::network::download::progress::ProgressEmitter;
use crate::services::theme::downloader::ThemeDownloader;

#[tauri::command]
pub async fn download_theme_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    downloader: State<'_, ThemeDownloader>,
    config: dwall::Config,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    let progress_emitter = ProgressEmitter::new(&window);
    downloader
        .download_and_extract(&config, theme_id, Some(&progress_emitter))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn cancel_theme_download_cmd(
    downloader: State<'_, ThemeDownloader>,
    theme_id: String,
) -> DwallSettingsResult<()> {
    downloader.cancel(&theme_id).await;
    Ok(())
}
