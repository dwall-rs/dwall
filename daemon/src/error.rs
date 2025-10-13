use crate::domain::geography::{GeolocationAccessError, CoordinateError};
use crate::domain::visual::ThemeError;
use crate::infrastructure::display::DisplayError;
use crate::infrastructure::display::WallpaperError;
use crate::infrastructure::platform::windows::RegistryError;

/// Application result type, used for unified error handling
pub type DwallResult<T> = std::result::Result<T, DwallError>;

/// Application global error type
///
/// Contains all possible error types that may occur during application execution
#[derive(Debug, thiserror::Error)]
pub enum DwallError {
    /// Input/Output error
    #[error("IO operation failed: {0}")]
    Io(#[from] std::io::Error),

    /// Windows API error
    #[error("Windows system call failed: {0}")]
    Windows(#[from] windows::core::Error),

    /// Theme-related error
    #[error("Theme processing error: {0}")]
    Theme(#[from] ThemeError),

    /// JSON serialization/deserialization error
    #[error("JSON processing failed: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Configuration-related error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// Registry related error
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    /// Null character error
    #[error("String contains null character: {0}")]
    NulError(#[from] std::ffi::NulError),

    /// Time offset error
    #[error("Unable to determine time offset: {0}")]
    TimeIndeterminateOffset(#[from] time::error::IndeterminateOffset),

    /// Monitor related error
    #[error("Monitor operation failed: {0}")]
    Monitor(#[from] DisplayError),

    /// Position related error
    #[error("Position error: {0}")]
    Position(#[from] CoordinateError),

    /// Geolocation access error
    #[error("Geolocation access error: {0}")]
    GeolocationAccess(#[from] GeolocationAccessError),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Wallpaper manager error
    #[error("Wallpaper operation failed: {0}")]
    WallpaperManager(#[from] WallpaperError),
}

/// Configuration-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error occurred: {0}")]
    Io(#[from] std::io::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    #[error("Configuration validation failed: {reason}")]
    Validation { reason: String },
}
