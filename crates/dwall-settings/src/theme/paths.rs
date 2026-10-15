//! Theme file system path resolution.

use std::path::PathBuf;

use dwall::DWALL_CACHE_DIR;

/// Resolves file system paths for theme storage
pub struct ThemePathResolver;

impl ThemePathResolver {
    /// Get the directory path for a theme
    pub fn theme_dir(config: &dwall::Config, theme_id: &str) -> PathBuf {
        config.themes_directory().join(theme_id)
    }

    /// Staging directory used while a theme is being downloaded and extracted.
    ///
    /// It is renamed onto [`theme_dir`](Self::theme_dir) only after a successful
    /// extraction, so an interrupted download never removes a working theme.
    pub fn staging_dir(config: &dwall::Config, theme_id: &str) -> PathBuf {
        config
            .themes_directory()
            .join(format!("{theme_id}.staging"))
    }

    /// Get the path for the theme zip file
    pub fn zip_path(config: &dwall::Config, theme_id: &str) -> PathBuf {
        config.themes_directory().join(format!("{theme_id}.zip"))
    }

    /// Get the path for the temporary download file
    pub fn temp_zip_path(config: &dwall::Config, theme_id: &str) -> PathBuf {
        config
            .themes_directory()
            .join(format!("{theme_id}.zip.tmp"))
    }

    /// Get the thumbnails directory for a theme
    pub fn thumbnails_dir(theme_id: &str) -> PathBuf {
        DWALL_CACHE_DIR.join("thumbnails").join(theme_id)
    }

    /// Build a cached image path
    pub fn cached_image_path(theme_id: &str, serial_number: u8, extension: &str) -> PathBuf {
        Self::thumbnails_dir(theme_id).join(format!("{serial_number}.{extension}"))
    }
}
