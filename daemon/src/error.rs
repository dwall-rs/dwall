use windows::Win32::Foundation::WIN32_ERROR;

use crate::color_mode::ColorModeRegistryError;

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
    Theme(#[from] crate::theme::ThemeError),

    /// JSON serialization/deserialization error
    #[error("JSON processing failed: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Configuration-related error
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    /// Color mode related error
    #[error("Color mode setting failed: {0}")]
    ColorMode(#[from] ColorModeRegistryError),

    /// Null character error
    #[error("String contains null character: {0}")]
    NulError(#[from] std::ffi::NulError),

    /// Time offset error
    #[error("Unable to determine time offset: {0}")]
    TimeIndeterminateOffset(#[from] time::error::IndeterminateOffset),

    /// Monitor related error
    #[error("Monitor operation failed: {0}")]
    Monitor(#[from] crate::monitor::error::MonitorError),
}

/// Registry operation related errors
///
/// Contains all possible errors that may occur during interaction with the Windows registry
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    /// Failed to open registry key
    #[error("Failed to open registry key: {0:?}")]
    Open(WIN32_ERROR),

    /// Failed to query registry value
    #[error("Failed to query registry value: {0:?}")]
    Query(WIN32_ERROR),

    /// Failed to set registry value
    #[error("Failed to set registry value: {0:?}")]
    Set(WIN32_ERROR),

    /// Failed to close registry handle
    #[error("Failed to close registry handle: {0:?}")]
    Close(WIN32_ERROR),

    /// Failed to delete registry key
    #[error("Failed to delete registry key: {0:?}")]
    Delete(WIN32_ERROR),
}
