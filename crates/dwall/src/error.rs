#[cfg(windows)]
use crate::platform::{DisplayError, RegistryError, WallpaperError};
use crate::solar::{CoordinateError, GeolocationAccessError};
use crate::theme::ThemeError;

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
    ThemeProcessing(#[from] ThemeError),

    /// JSON serialization/deserialization error
    #[error("JSON processing failed: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Configuration-related error
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[cfg(windows)]
    /// Registry related error
    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    /// Null character error
    #[error("String contains null character: {0}")]
    NulError(#[from] std::ffi::NulError),

    /// Monitor related error
    #[error("Monitor operation failed: {0}")]
    Monitor(#[from] DisplayError),

    /// Position related error
    #[error("Position error: {0}")]
    Position(#[from] CoordinateError),

    /// Geolocation access error
    #[error("Geolocation access error: {0}")]
    GeolocationAccess(#[from] GeolocationAccessError),

    /// Wallpaper manager error
    #[error("Wallpaper operation failed: {0}")]
    WallpaperManager(#[from] WallpaperError),

    /// A WinRT asynchronous operation did not complete within its budget
    #[error("WinRT operation did not complete within {0:?}")]
    AsyncTimeout(std::time::Duration),

    #[error(transparent)]
    DateTime(#[from] time::Error),

    #[error(transparent)]
    Logger(#[from] log::SetLoggerError),
}

impl DwallError {
    /// Whether the daemon has no location to work from because of the user's settings.
    ///
    /// Only automatic coordinates can produce this, and it is a decision rather than a
    /// failure to retry: the daemon must not guess a position (a fallback to the last
    /// one it saw would quietly track the wrong sun for a machine that moved), so it
    /// stops and lets the log, the exit code and the settings window say why.
    ///
    /// The transient failures that used to kill the daemon are *not* included: a
    /// timeout or a busy sensor is retried with a backoff, because a daemon that
    /// disappears at sign-in, before the location service is ready, is worse than one
    /// that keeps trying.
    pub fn location_unavailable(&self) -> bool {
        /// `E_ACCESSDENIED`, which is what the WinRT calls report when location access
        /// is switched off for the app (`HRESULT_FROM_WIN32(ERROR_ACCESS_DENIED)`).
        const E_ACCESSDENIED: u32 = 0x8007_0005;

        match self {
            Self::GeolocationAccess(GeolocationAccessError::Denied)
            | Self::GeolocationAccess(GeolocationAccessError::Unspecified) => true,
            Self::Windows(e) => e.code().0 as u32 == E_ACCESSDENIED,
            _ => false,
        }
    }
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
