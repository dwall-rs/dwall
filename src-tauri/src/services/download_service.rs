//! Download service module
//!
//! This module provides high-level download functionality by coordinating with the infrastructure.

use dwall::Config;
use tauri::{Runtime, State, WebviewWindow};

use crate::{
    error::DwallSettingsResult,
    infrastructure::network::download::{ProgressEmitter, ThemeDownloader, ThemeExtractor},
};

/// Download and extract a theme
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
