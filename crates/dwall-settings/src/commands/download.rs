//! Download commands

use tauri::State;

use crate::error::DwallSettingsResult;
use crate::filesystem::{validate_theme_id, validate_theme_roots};
use crate::network::progress::ProgressEmitter;
use crate::theme::downloader::ThemeDownloader;

#[tauri::command]
pub async fn download_theme_cmd<R: tauri::Runtime>(
    window: tauri::WebviewWindow<R>,
    downloader: State<'_, ThemeDownloader>,
    config: dwall::Config,
    theme_id: &str,
) -> DwallSettingsResult<()> {
    // The id becomes a directory name and the downloader clears it before writing.
    validate_theme_id(theme_id)?;

    // The roots become the parent directories of `remove_dir_all` and the
    // extraction target, so they get the same validation as every other command
    // that acts on a renderer-supplied configuration.
    validate_theme_roots(&config)?;

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
