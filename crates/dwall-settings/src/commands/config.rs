//! Configuration commands

use dwall::config::{ConfigReader, ConfigWriter};
use tauri::State;

use crate::error::DwallSettingsResult;
use crate::filesystem::validate_directory;
use crate::theme::downloader::ThemeDownloader;
use crate::thumbnail::ThumbnailCache;

#[tauri::command]
pub async fn read_config_file() -> DwallSettingsResult<dwall::Config> {
    let config_path = dwall::DWALL_CONFIG_DIR.join("config.toml");
    ConfigReader::read_from_path(&config_path).map_err(Into::into)
}

#[tauri::command]
pub async fn write_config_file(
    downloader: State<'_, ThemeDownloader>,
    thumbnail_cache: State<'_, ThumbnailCache>,
    config: dwall::Config,
) -> DwallSettingsResult<()> {
    // These paths drive recursive filesystem operations, so validate what the
    // renderer sent before persisting it.
    validate_directory(config.themes_directory(), "themes_directory")?;
    validate_directory(
        config.customized_themes_directory(),
        "customized_themes_directory",
    )?;

    debug!(config = ?config, "Writing config file");
    let config_path = dwall::DWALL_CONFIG_DIR.join("config.toml");
    let writer = ConfigWriter;
    writer.write_to_path(&config_path, &config)?;

    // The proxy lives inside the HTTP client, so a changed network setting has to
    // be pushed into it; otherwise it only takes effect after a restart.
    if let Err(e) = downloader.reconfigure(config.network()) {
        warn!(error = %e, "Failed to apply the network configuration to the HTTP client");
    }
    if let Err(e) = thumbnail_cache.reconfigure(config.network()) {
        warn!(error = %e, "Failed to apply the network configuration to the thumbnail client");
    }

    Ok(())
}
