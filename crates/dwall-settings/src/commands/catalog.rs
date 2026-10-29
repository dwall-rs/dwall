//! Theme catalog and wallpaper matching commands.

use std::path::{Path, PathBuf};

use dwall::ThemeValidator;
use dwall::config::{Config, ConfigReader, ImageFormat};
use dwall::theme::{get_theme_directory_path, read_solar_angles};
use dwall::theme_manifest::ThemeManifest;
use dwall::wallpaper::WallpaperSelector;
use dwall::{DWALL_CONFIG_DIR, SolarAngle};
use serde::Serialize;

use crate::error::DwallSettingsResult;
use crate::filesystem::validate_theme_id;

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
    /// Whether the theme's frames are on disk.
    ///
    /// A catalog theme that is not installed yet is what the library offers to
    /// download; it is computed from the same lookup [`get_theme_wallpapers`] uses,
    /// so the library and the stage always agree on which themes are missing.
    pub installed: bool,
}

/// The configuration file the commands read, in the user's profile.
fn default_config_path() -> PathBuf {
    DWALL_CONFIG_DIR.join("config.toml")
}

/// Reads the configuration, or `None` when it is missing or unreadable.
///
/// A missing configuration is not an error here: the catalog is still the list of
/// themes that *could* be installed.
fn read_config(config_path: &Path) -> Option<Config> {
    match ConfigReader::read_from_path(config_path) {
        Ok(config) => Some(config),
        Err(e) => {
            // The catalog then lists only what could be downloaded; saying so keeps a
            // broken configuration from looking like an empty one.
            warn!(
                path = %config_path.display(),
                error = %e,
                "Could not read the configuration"
            );
            None
        }
    }
}

/// List the themes available for download, plus the installed custom themes.
///
/// Runs off the main thread: the catalog reads the configuration and stats a directory
/// per entry, and the library calls it every time it opens.
#[tauri::command]
pub async fn get_theme_catalog() -> Vec<CatalogTheme> {
    match tauri::async_runtime::spawn_blocking(|| {
        catalog_from(read_config(&default_config_path()).as_ref())
    })
    .await
    {
        Ok(catalog) => catalog,
        Err(e) => {
            error!(error = %e, "Failed to build the theme catalog");
            Vec::new()
        }
    }
}

/// The catalog for an already-read configuration.
///
/// The configuration is a parameter rather than read per entry: it used to be read
/// and parsed once for *every* catalog theme, which is a file open plus a TOML parse
/// per entry on a command the library calls on every open.
fn catalog_from(config: Option<&Config>) -> Vec<CatalogTheme> {
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
                installed: is_installed(name, config),
            }
        })
        .collect();

    themes.extend(custom_catalog_themes(config));
    themes
}

/// Installed custom themes (`theme.toml`, schema 1) as catalog entries.
fn custom_catalog_themes(config: Option<&Config>) -> Vec<CatalogTheme> {
    let Some(config) = config else {
        return Vec::new();
    };
    let directory = config.customized_themes_directory();
    let entries = match std::fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(e) => {
            // A customized directory that does not exist yet is normal on a fresh
            // install, so this is not a warning.
            debug!(
                dir = %directory.display(),
                error = %e,
                "Could not list the customized themes directory"
            );
            return Vec::new();
        }
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
            // A custom theme whose frame images are missing cannot be applied; listing
            // it as installed would leave the user with no download button and no
            // explanation. Skip it like the customized-theme listing does.
            if let Err(e) = ThemeValidator::validate_uncached(
                config.customized_themes_directory(),
                &id,
                true,
                manifest.image_format(),
            ) {
                warn!(theme_id = %id, error = %e, "Skipping an incomplete custom theme");
                return None;
            }
            Some(CatalogTheme {
                id,
                name: manifest.theme.name.clone(),
                source: "custom".to_string(),
                thumbnails: local_thumbnails(&dir, manifest.image_format()),
                // The directory, its manifest and its frames are what make a custom
                // theme exist.
                installed: true,
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

fn resolve_theme_dir(
    theme_id: &str,
    config: Option<&Config>,
) -> DwallSettingsResult<Option<PathBuf>> {
    let Some(config) = config else {
        return Ok(None);
    };

    let (dir, _is_customized) = get_theme_directory_path(config, theme_id);
    Ok(Some(dir))
}

fn installed_solar_angles(
    theme_id: &str,
    config: Option<&Config>,
) -> DwallSettingsResult<Vec<SolarAngle>> {
    // The id arrives from the renderer and becomes a path component below; the write
    // path already validates it, and a read must not be the way around that.
    validate_theme_id(theme_id)?;

    let Some(dir) = resolve_theme_dir(theme_id, config)? else {
        return Ok(Vec::new());
    };
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    Ok(read_solar_angles(&dir)?)
}

/// Whether a theme is complete enough for the daemon to apply.
///
/// Frames alone are not enough: a directory can have a valid `solar.json` and a
/// missing frame image. The daemon refuses that theme at apply time, but the library
/// used to call it installed and hid the only download entry that could repair it.
fn is_installed(theme_id: &str, config: Option<&Config>) -> bool {
    let Some(config) = config else {
        return false;
    };

    let (directory, is_customized) = get_theme_directory_path(config, theme_id);
    if !directory.is_dir() {
        return false;
    }

    let root = if is_customized {
        config.customized_themes_directory()
    } else {
        config.themes_directory()
    };

    ThemeValidator::validate_uncached(root, theme_id, is_customized, config.image_format()).is_ok()
}

/// Wallpapers (with their target solar angles) for an installed theme.
///
/// Returns an empty list when the theme is not installed.
#[tauri::command]
pub async fn get_theme_wallpapers(theme_id: String) -> DwallSettingsResult<Vec<SolarAngle>> {
    tauri::async_runtime::spawn_blocking(move || {
        installed_solar_angles(&theme_id, read_config(&default_config_path()).as_ref())
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

/// Position (in the theme's `solar.json` array) of the entry whose target solar
/// angle is closest to the given position.
///
/// A theme may reuse one image for several sun positions, so the closest
/// *entry* — not just its image index — is what the UI needs to highlight.
#[tauri::command]
pub async fn match_wallpaper(
    theme_id: String,
    altitude: f64,
    azimuth: f64,
) -> DwallSettingsResult<Option<usize>> {
    tauri::async_runtime::spawn_blocking(move || -> DwallSettingsResult<Option<usize>> {
        let angles =
            installed_solar_angles(&theme_id, read_config(&default_config_path()).as_ref())?;
        Ok(WallpaperSelector::find_closest_entry(
            &angles, altitude, azimuth,
        ))
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch profile with the two theme directories a configuration names.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_catalog_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("themes")).unwrap();
        std::fs::create_dir_all(dir.join("customize")).unwrap();

        dir
    }

    /// A configuration rooted in `dir`, built through `serde` because the fields are
    /// private.
    fn config_at(dir: &Path) -> Config {
        let json = format!(
            r#"{{"themes_directory":{},"customized_themes_directory":{}}}"#,
            serde_json::to_string(&dir.join("themes").to_string_lossy()).unwrap(),
            serde_json::to_string(&dir.join("customize").to_string_lossy()).unwrap(),
        );

        serde_json::from_str(&json).unwrap()
    }

    /// A legacy theme: `solar.json` plus the image its single frame names.
    fn write_legacy_theme(themes_directory: &Path, theme_id: &str) {
        let theme_dir = themes_directory.join(theme_id);
        std::fs::create_dir_all(theme_dir.join("jpg")).unwrap();
        std::fs::write(
            theme_dir.join("solar.json"),
            r#"[{"index": 0, "altitude": 10.0, "azimuth": 20.0}]"#,
        )
        .unwrap();
        std::fs::write(theme_dir.join("jpg").join("1.jpg"), "image").unwrap();
    }

    fn entry<'a>(catalog: &'a [CatalogTheme], id: &str) -> Option<&'a CatalogTheme> {
        catalog.iter().find(|theme| theme.id == id)
    }

    /// The id becomes a path component; a renderer-supplied value that climbs out of
    /// the themes directory has to be refused, exactly as the write path refuses it.
    #[test]
    fn installed_solar_angles_rejects_a_traversing_theme_id() {
        let dir = scratch("traversal");

        let error = installed_solar_angles("../escape", Some(&config_at(&dir))).unwrap_err();

        assert!(
            error.to_string().contains("Invalid theme id"),
            "unexpected error: {error}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_installed_catalog_theme_is_marked_installed_and_the_others_are_not() {
        let dir = scratch("installed");
        write_legacy_theme(&dir.join("themes"), "Big Sur");

        let catalog = catalog_from(Some(&config_at(&dir)));

        assert!(entry(&catalog, "Big Sur").unwrap().installed);
        assert!(
            !entry(&catalog, "Catalina").unwrap().installed,
            "a theme that is not on disk is what the library offers to download"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A directory with images but no frames cannot be applied, so it must not be
    /// reported as installed — the library would otherwise offer no download and the
    /// stage would show an empty theme.
    #[test]
    fn a_theme_without_frames_is_not_installed() {
        let dir = scratch("no_frames");
        let theme_dir = dir.join("themes").join("Big Sur");
        std::fs::create_dir_all(theme_dir.join("jpg")).unwrap();
        std::fs::write(theme_dir.join("jpg").join("1.jpg"), "image").unwrap();

        let catalog = catalog_from(Some(&config_at(&dir)));

        assert!(!entry(&catalog, "Big Sur").unwrap().installed);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A theme with frames but no image for a frame cannot be applied. It must not be
    /// reported as installed, or the library hides the download that could repair it.
    #[test]
    fn a_theme_with_a_missing_frame_image_is_not_installed() {
        let dir = scratch("missing_image");
        write_legacy_theme(&dir.join("themes"), "Big Sur");
        std::fs::remove_file(dir.join("themes").join("Big Sur").join("jpg").join("1.jpg")).unwrap();

        let catalog = catalog_from(Some(&config_at(&dir)));

        assert!(!entry(&catalog, "Big Sur").unwrap().installed);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_installed_custom_theme_becomes_a_catalog_entry() {
        let dir = scratch("custom");
        let theme_dir = dir.join("customize").join("My Theme");
        std::fs::create_dir_all(theme_dir.join("images")).unwrap();
        std::fs::write(
            theme_dir.join("theme.toml"),
            r#"schema = 1

[theme]
id = "my-theme"
name = "My Theme"
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
"#,
        )
        .unwrap();
        std::fs::write(theme_dir.join("images").join("1.png"), "image").unwrap();

        let catalog = catalog_from(Some(&config_at(&dir)));

        let custom = entry(&catalog, "My Theme").expect("the custom theme is listed");
        assert_eq!(custom.source, "custom");
        assert!(custom.installed);
        assert_eq!(
            custom.thumbnails,
            vec![
                theme_dir
                    .join("images")
                    .join("1.png")
                    .to_string_lossy()
                    .into_owned()
            ]
        );

        // The manifest's name is what the library shows, not the directory name.
        assert_eq!(custom.name, "My Theme");

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Without a configuration the catalog is still the list of downloadable themes:
    /// a first run has nothing installed yet.
    #[test]
    fn without_a_configuration_only_the_downloadable_themes_are_listed() {
        let catalog = catalog_from(None);

        assert_eq!(catalog.len(), THEME_CATALOG.len());
        assert!(catalog.iter().all(|theme| !theme.installed));
        assert!(catalog.iter().all(|theme| theme.source == "catalog"));
        assert_eq!(
            entry(&catalog, "Ventura Graphic").unwrap().thumbnails.len(),
            5,
            "the thumbnail count comes from the catalog table"
        );
    }

    /// A configuration that cannot be parsed is reported and treated as absent, so
    /// the catalog falls back to the downloadable list instead of failing.
    #[test]
    fn an_unreadable_configuration_is_treated_as_absent() {
        let dir = scratch("unreadable_config");
        let path = dir.join("config.toml");
        std::fs::write(&path, "not = = toml").unwrap();

        assert!(read_config(&path).is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── local_thumbnails ────────────────────────────────────────────────────

    /// Thumbnails are ordered by their numeric file stem, not lexically, so image 10
    /// follows image 2. A file that is not an image, or whose stem is not a number,
    /// is not a thumbnail, and the extension matches regardless of case.
    #[test]
    fn local_thumbnails_are_ordered_numerically_and_filtered() {
        let dir = scratch("thumbnail_order");
        let theme_dir = dir.join("themes").join("Big Sur");
        let thumbnails = theme_dir.join("thumbnails");
        std::fs::create_dir_all(&thumbnails).unwrap();
        for name in ["10.JPG", "2.jpg", "1.jpg", "notes.txt", "cover.jpg"] {
            std::fs::write(thumbnails.join(name), "image").unwrap();
        }

        let names: Vec<String> = local_thumbnails(&theme_dir, &ImageFormat::Jpeg)
            .iter()
            .map(|path| {
                Path::new(path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();

        assert_eq!(names, vec!["1.jpg", "2.jpg", "10.JPG"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// thumbnails/ wins over images/ when both exist, and images/ is the fallback for
    /// a theme that ships no thumbnails.
    #[test]
    fn local_thumbnails_prefer_the_thumbnails_directory_then_fall_back() {
        let dir = scratch("thumbnail_fallback");
        let theme_dir = dir.join("themes").join("Big Sur");
        std::fs::create_dir_all(theme_dir.join("thumbnails")).unwrap();
        std::fs::create_dir_all(theme_dir.join("images")).unwrap();
        std::fs::write(theme_dir.join("images").join("1.jpg"), "image").unwrap();

        let from_images = local_thumbnails(&theme_dir, &ImageFormat::Jpeg);
        assert_eq!(from_images.len(), 1);
        assert!(from_images[0].contains("images"), "{}", from_images[0]);

        std::fs::write(theme_dir.join("thumbnails").join("2.jpg"), "image").unwrap();

        let from_thumbnails = local_thumbnails(&theme_dir, &ImageFormat::Jpeg);
        assert_eq!(from_thumbnails.len(), 1);
        assert!(
            from_thumbnails[0].contains("thumbnails"),
            "{}",
            from_thumbnails[0]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
