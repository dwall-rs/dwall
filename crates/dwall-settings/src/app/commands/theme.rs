//! Theme commands

use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

use dwall::config::ImageFormat;

use crate::domain::theme::validator::ThemeValidator;
use crate::domain::theme::{CustomizedThemeMetadata, types::CustomizedTheme};
use crate::error::DwallSettingsResult;
use crate::infrastructure::filesystem::ops::{find_files_in_dir, list_subdirectories};
use crate::services::theme::applier::ThemeApplier;
use crate::services::theme::status::ThemeStatusProvider;

#[tauri::command]
pub async fn validate_theme_cmd(
    themes_directory: &Path,
    theme_id: &str,
    is_customized: bool,
    image_format: ImageFormat,
) -> DwallSettingsResult<()> {
    ThemeValidator::validate(themes_directory, theme_id, is_customized, &image_format)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_applied_theme_id_cmd(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
    ThemeStatusProvider::get_current_theme_id(monitor_id)
}

#[tauri::command]
pub async fn apply_theme_cmd(config: dwall::Config) -> DwallSettingsResult<()> {
    ThemeApplier::apply(config).await
}

#[tauri::command]
pub async fn get_customized_themes_cmd(
    customized_themes_directory: PathBuf,
) -> DwallSettingsResult<Vec<CustomizedTheme>> {
    if !customized_themes_directory.exists() {
        return Ok(Vec::new());
    }

    let subdirs = list_subdirectories(&customized_themes_directory).await?;
    debug!(subdirs = ?subdirs, "Found subdirectories");

    let mut themes = Vec::with_capacity(subdirs.len());
    for subdir in subdirs
        .into_iter()
        .filter(|p| p.components().next_back() != Some(Component::Normal(OsStr::new("backup"))))
    {
        let metadata_file = subdir.join("metadata.toml");
        let metadata_content = std::fs::read_to_string(&metadata_file)
            .inspect_err(|e| error!(error = ?e, "Failed to read metadata.toml"))?;
        let metadata: CustomizedThemeMetadata = toml::from_str(&metadata_content)
            .inspect_err(|e| error!(error = ?e, "Failed to parse metadata"))?;

        let images = find_files_in_dir(&subdir.join("images"), metadata.image_format.as_str())
            .await
            .inspect_err(|e| error!(error = ?e, "Failed to find images in directory"))?;
        debug!(images = images.len(), "Found images");

        let thumbnails = find_files_in_dir(&subdir.join("thumbnails"), "avif")
            .await
            .inspect_err(|e| error!(error = ?e, "Failed to find avif files in directory"))
            .ok();
        debug!(
            thumbnails = thumbnails.as_deref().map_or(0, |t| t.len()),
            "Found thumbnails"
        );

        if let Some(thumbnails) = &thumbnails
            && images.len() != thumbnails.len()
        {
            error!(
                images = images.len(),
                thumbnails = thumbnails.len(),
                "Invalid subject: number of images and thumbnails are not equal"
            );
            continue;
        }

        themes.push(CustomizedTheme {
            id: format!(
                "{}-{}-v{}",
                metadata.theme_name.replace(" ", "-"),
                metadata.author,
                metadata.version
            ),
            directory: subdir,
            thumbnails: thumbnails.unwrap_or(images),
            metadata,
        });
    }

    info!("Found {} customized themes", themes.len());

    Ok(themes)
}
