//! Theme catalog and wallpaper matching commands.

use std::path::{Path, PathBuf};

use dwall::config::{ConfigReader, ImageFormat};
use dwall::theme::{get_theme_directory_path, read_solar_angles};
use dwall::theme_manifest::ThemeManifest;
use dwall::wallpaper::WallpaperSelector;
use dwall::{DWALL_CONFIG_DIR, SolarAngle};
use serde::Serialize;

use crate::error::DwallSettingsResult;

/// Base URL for theme thumbnails in the dwall-assets repository.
const THUMBNAILS_BASE_URL: &str =
    "https://github.com/dwall-rs/dwall-assets/raw/refs/heads/main/thumbnails/";

/// Static catalog of downloadable themes: `(display name, thumbnail count)`.
const THEME_CATALOG: &[(&str, u32)] = &[
    ("Big Sur", 8),
    ("Big Sur 1", 16),
    ("Catalina", 8),
    ("Earth ISS", 16),
    ("Earth View", 16),
    ("Minya Konka", 24),
    ("Mojave", 16),
    ("Monterey Bay 1", 16),
    ("Monterey Graphic", 8),
    ("Solar Gradients", 16),
    ("The Beach", 8),
    ("The Cliffs", 8),
    ("The Desert", 8),
    ("The Lake", 8),
    ("Ventura Graphic", 5),
];

/// A theme available for download (built-in) or already installed (custom).
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(export))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogTheme {
    pub id: String,
    pub name: String,
    /// `"catalog"` (downloadable, remote thumbnails) or `"custom"` (installed
    /// under the customized themes directory, local thumbnail paths).
    pub source: String,
    /// Remote URLs for catalog themes; absolute local paths for custom themes.
    pub thumbnails: Vec<String>,
}

/// List the themes available for download, plus the installed custom themes.
#[tauri::command]
pub fn get_theme_catalog() -> Vec<CatalogTheme> {
    let mut themes: Vec<CatalogTheme> = THEME_CATALOG
        .iter()
        .map(|&(name, count)| {
            let folder = name.replace(' ', "");
            let thumbnails = (1..=count)
                .map(|i| format!("{THUMBNAILS_BASE_URL}{folder}/{i}.avif"))
                .collect();
            CatalogTheme {
                id: name.to_string(),
                name: name.to_string(),
                source: "catalog".to_string(),
                thumbnails,
            }
        })
        .collect();

    themes.extend(custom_catalog_themes());
    themes
}

/// Installed custom themes (`theme.toml`, schema 1) as catalog entries.
fn custom_catalog_themes() -> Vec<CatalogTheme> {
    let Ok(config) = ConfigReader::read_from_path(&DWALL_CONFIG_DIR.join("config.toml")) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(config.customized_themes_directory()) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|entry| {
            let dir = entry.path();
            let id = dir.file_name()?.to_str()?.to_owned();
            if id == "backup" {
                return None;
            }
            let manifest = ThemeManifest::read(&dir).ok()?;
            Some(CatalogTheme {
                id,
                name: manifest.theme.name.clone(),
                source: "custom".to_string(),
                thumbnails: local_thumbnails(&dir, manifest.image_format()),
            })
        })
        .collect()
}

/// Local thumbnail paths for a custom theme, ordered by image index.
///
/// Prefers `thumbnails/`, falling back to `images/`; files are `<index + 1>.<ext>`.
fn local_thumbnails(theme_dir: &Path, image_format: &ImageFormat) -> Vec<String> {
    for subdir in ["thumbnails", "images"] {
        let dir = theme_dir.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        let mut files: Vec<(u64, PathBuf)> = entries
            .flatten()
            .filter_map(|entry| {
                let path = entry.path();
                let extension = path.extension()?.to_str()?;
                if !path.is_file() || !extension.eq_ignore_ascii_case(image_format.as_str()) {
                    return None;
                }
                let index = path.file_stem()?.to_str()?.parse::<u64>().ok()?;
                Some((index, path))
            })
            .collect();

        if !files.is_empty() {
            files.sort_by_key(|(index, _)| *index);
            return files
                .into_iter()
                .map(|(_, path)| path.to_string_lossy().into_owned())
                .collect();
        }
    }
    Vec::new()
}

fn resolve_theme_dir(theme_id: &str) -> DwallSettingsResult<PathBuf> {
    let config = ConfigReader::read_from_path(&DWALL_CONFIG_DIR.join("config.toml"))?;
    let (dir, _is_customized) = get_theme_directory_path(&config, theme_id);
    Ok(dir)
}

fn installed_solar_angles(theme_id: &str) -> DwallSettingsResult<Vec<SolarAngle>> {
    let dir = resolve_theme_dir(theme_id)?;
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    Ok(read_solar_angles(&dir)?)
}

/// Wallpapers (with their target solar angles) for an installed theme.
///
/// Returns an empty list when the theme is not installed.
#[tauri::command]
pub fn get_theme_wallpapers(theme_id: String) -> DwallSettingsResult<Vec<SolarAngle>> {
    installed_solar_angles(&theme_id)
}

/// Position (in the theme's `solar.json` array) of the entry whose target solar
/// angle is closest to the given position.
///
/// A theme may reuse one image for several sun positions, so the closest
/// *entry* — not just its image index — is what the UI needs to highlight.
#[tauri::command]
pub fn match_wallpaper(
    theme_id: String,
    altitude: f64,
    azimuth: f64,
) -> DwallSettingsResult<Option<usize>> {
    let angles = installed_solar_angles(&theme_id)?;
    Ok(WallpaperSelector::find_closest_entry(
        &angles, altitude, azimuth,
    ))
}
