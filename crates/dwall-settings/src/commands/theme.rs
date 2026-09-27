//! Theme commands

use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

use dwall::config::ImageFormat;

use crate::error::DwallSettingsResult;
use crate::filesystem::{find_files_in_dir, list_subdirectories};
use crate::theme::applier::ThemeApplier;
use crate::theme::status::ThemeStatusProvider;
use crate::theme::validator::ThemeValidator;
use crate::theme::{CustomizedThemeMetadata, types::CustomizedTheme};

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
        // The directory name is the theme id the daemon resolves against.
        let Some(id) = subdir
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
        else {
            warn!(directory = %subdir.display(), "Skipping custom theme with non-UTF-8 name");
            continue;
        };

        let manifest = match dwall::theme_manifest::ThemeManifest::read(&subdir) {
            Ok(manifest) => manifest,
            Err(e) => {
                warn!(theme_id = %id, error = %e, "Skipping invalid custom theme");
                continue;
            }
        };

        let image_format = manifest.image_format().clone();
        let images = find_files_in_dir(&subdir.join("images"), image_format.as_str())
            .await
            .inspect_err(|e| error!(error = ?e, "Failed to find images in directory"))?;
        debug!(images = images.len(), "Found images");

        // Thumbnails are optional: use them when present, else fall back to images.
        let mut thumbnails = find_files_in_dir(&subdir.join("thumbnails"), image_format.as_str())
            .await
            .unwrap_or_default();
        if thumbnails.is_empty() {
            thumbnails = find_files_in_dir(&subdir.join("thumbnails"), "webp")
                .await
                .unwrap_or_default();
        }
        if thumbnails.is_empty() {
            thumbnails = images;
        }

        themes.push(CustomizedTheme {
            id,
            directory: subdir,
            thumbnails,
            metadata: CustomizedThemeMetadata {
                image_format,
                theme_name: manifest.theme.name.clone(),
                author: manifest.theme.author.clone(),
                version: manifest.theme.version,
            },
        });
    }

    info!("Found {} customized themes", themes.len());

    Ok(themes)
}
