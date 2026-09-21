//! Theme catalog and wallpaper matching commands.

use std::path::PathBuf;

use dwall::config::ConfigReader;
use dwall::theme::{get_theme_directory_path, read_solar_angles, wallpaper_image_path};
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

/// A theme available for download.
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(export))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogTheme {
    pub id: String,
    pub name: String,
    pub thumbnails: Vec<String>,
}

/// List the themes available for download.
#[tauri::command]
pub fn get_theme_catalog() -> Vec<CatalogTheme> {
    THEME_CATALOG
        .iter()
        .map(|&(name, count)| {
            let folder = name.replace(' ', "");
            let thumbnails = (1..=count)
                .map(|i| format!("{THUMBNAILS_BASE_URL}{folder}/{i}.avif"))
                .collect();
            CatalogTheme {
                id: name.to_string(),
                name: name.to_string(),
                thumbnails,
            }
        })
        .collect()
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

/// On-disk path of a theme wallpaper image, or `None` when it is missing.
#[tauri::command]
pub fn get_theme_wallpaper_path(
    theme_id: String,
    index: u8,
) -> DwallSettingsResult<Option<String>> {
    let config = ConfigReader::read_from_path(&DWALL_CONFIG_DIR.join("config.toml"))?;
    let (dir, is_customized) = get_theme_directory_path(&config, &theme_id);
    if !dir.is_dir() {
        return Ok(None);
    }
    let path = wallpaper_image_path(&config, &dir, index, is_customized);
    Ok(path.exists().then(|| path.display().to_string()))
}

/// Index of the wallpaper whose target solar angle is closest to the given position.
#[tauri::command]
pub fn match_wallpaper(
    theme_id: String,
    altitude: f64,
    azimuth: f64,
) -> DwallSettingsResult<Option<u8>> {
    let angles = installed_solar_angles(&theme_id)?;
    Ok(WallpaperSelector::find_closest_image(
        &angles, altitude, azimuth,
    ))
}
