//! Theme error types

/// Comprehensive error handling for solar theme-related operations
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Theme directory '{0}' does not exist")]
    ThemeDirectoryNotFound(String),
    #[error("Default theme is missing or not configured")]
    DefaultThemeMissing,
    #[error("Solar configuration file 'solar.json' is missing in theme directory")]
    SolarConfigurationMissing,
    #[error("Image files do not match solar configuration: expected {expected}, found {found}")]
    ImageSolarConfigurationMismatch { expected: usize, found: usize },
    #[error("Wallpaper image file '{path}' does not exist")]
    WallpaperImageMissing { path: String },
    #[error("No monitor-specific wallpaper configurations found")]
    MonitorWallpaperConfigurationMissing,
}
