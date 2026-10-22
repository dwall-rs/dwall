//! Theme commands

use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

use dwall::config::ImageFormat;

use crate::error::DwallSettingsResult;
use crate::filesystem::{
    find_files_in_dir, list_subdirectories, validate_directory, validate_theme_id,
};
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
    // Both values arrive from the renderer and are joined into a path below.
    validate_directory(themes_directory, "themes_directory")?;
    validate_theme_id(theme_id)?;

    ThemeValidator::validate(themes_directory, theme_id, is_customized, &image_format)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn get_applied_theme_id_cmd(monitor_id: &str) -> DwallSettingsResult<Option<String>> {
    ThemeStatusProvider::get_current_theme_id(monitor_id)
}

#[tauri::command]
pub async fn apply_theme_cmd(
    downloader: tauri::State<'_, crate::theme::downloader::ThemeDownloader>,
    config: dwall::Config,
) -> DwallSettingsResult<()> {
    let network = config.network().cloned();
    ThemeApplier::apply(config).await?;

    // Applying also writes the config, so keep the HTTP client in sync here too.
    if let Err(e) = downloader.reconfigure(network.as_ref()) {
        warn!(error = %e, "Failed to apply the network configuration to the HTTP client");
    }

    Ok(())
}

#[tauri::command]
pub async fn get_customized_themes_cmd(
    customized_themes_directory: PathBuf,
) -> DwallSettingsResult<Vec<CustomizedTheme>> {
    // The directory arrives from the renderer and is walked below.
    validate_directory(&customized_themes_directory, "customized_themes_directory")?;

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
        // A custom theme whose `images/` directory is missing or unreadable cannot be
        // applied, so skip it exactly like an invalid manifest: one broken directory
        // must not hide every other custom theme in the list.
        let images = match find_files_in_dir(&subdir.join("images"), image_format.as_str()).await {
            Ok(images) => images,
            Err(e) => {
                warn!(theme_id = %id, error = %e, "Skipping custom theme whose images could not be listed");
                continue;
            }
        };
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch `customized_themes_directory`.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_theme_cmd_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// Writes a schema-valid `theme.toml` under `theme_dir`.
    fn write_manifest(theme_dir: &Path, id: &str, name: &str) {
        std::fs::create_dir_all(theme_dir).unwrap();
        std::fs::write(
            theme_dir.join("theme.toml"),
            format!(
                r#"schema = 1

[theme]
id = "{id}"
name = "{name}"
author = "tester"
version = 1
created_at = 1700000000
image_format = "png"

[solar]
mode = "native"

[[solar.frames]]
index = 0
altitude = 10.0
azimuth = 20.0
"#
            ),
        )
        .unwrap();
    }

    /// One custom theme whose `images/` directory cannot be read must not fail the whole
    /// listing: the settings window would otherwise show no custom theme at all.
    #[tokio::test]
    async fn a_broken_custom_theme_does_not_hide_the_others() {
        let dir = scratch("broken_listing");

        let good = dir.join("Good Theme");
        write_manifest(&good, "good-theme", "Good Theme");
        std::fs::create_dir_all(good.join("images")).unwrap();
        std::fs::write(good.join("images").join("1.png"), "image").unwrap();

        // Schema-valid manifest, but no `images/` directory to list.
        let broken = dir.join("Broken Theme");
        write_manifest(&broken, "broken-theme", "Broken Theme");

        let themes = get_customized_themes_cmd(dir.clone()).await.unwrap();

        let ids: Vec<&str> = themes.iter().map(|theme| theme.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["Good Theme"],
            "the broken theme is skipped, not fatal"
        );
        assert_eq!(themes[0].thumbnails.len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Both values arrive from the renderer and are joined into a path below.
    #[tokio::test]
    async fn validate_theme_cmd_refuses_a_traversing_id_and_a_relative_directory() {
        let dir = scratch("validation");

        assert!(
            validate_theme_cmd(&dir, "../escape", false, ImageFormat::Jpeg)
                .await
                .is_err(),
            "a traversing id must not resolve under the themes directory"
        );
        assert!(
            validate_theme_cmd(Path::new("relative"), "Big Sur", false, ImageFormat::Jpeg)
                .await
                .is_err(),
            "a relative root must be refused"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The customized-themes command walks the directory the renderer names.
    #[tokio::test]
    async fn get_customized_themes_cmd_refuses_a_relative_directory() {
        assert!(
            get_customized_themes_cmd(PathBuf::from("relative"))
                .await
                .is_err()
        );
    }
}
