//! Thumbnail cache commands

use tauri::State;

use crate::error::DwallSettingsResult;
use crate::filesystem::validate_theme_id;
use crate::thumbnail::{ThumbnailCache, clear_thumbnail_cache, get_or_save_cached_thumbnails};

#[tauri::command]
pub async fn get_or_save_cached_thumbnails_cmd(
    cache: State<'_, ThumbnailCache>,
    theme_id: &str,
    serial_number: u8,
    url: &str,
) -> DwallSettingsResult<std::path::PathBuf> {
    validate_theme_id(theme_id)?;

    get_or_save_cached_thumbnails(&cache, theme_id, serial_number, url).await
}

#[tauri::command]
pub async fn clear_thumbnail_cache_cmd() -> DwallSettingsResult<u64> {
    clear_thumbnail_cache().await
}
