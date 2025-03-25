use windows::Win32::Foundation::WIN32_ERROR;

use crate::color_mode::ColorModeRegistryError;

pub type DwallResult<T> = std::result::Result<T, DwallError>;

#[derive(Debug, thiserror::Error)]
pub enum DwallError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Windows(#[from] windows::core::Error),
    #[error(transparent)]
    Theme(#[from] crate::theme::ThemeError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    ColorMode(#[from] ColorModeRegistryError),
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
    #[error(transparent)]
    TimeIndeterminateOffset(#[from] time::error::IndeterminateOffset),
    #[error(transparent)]
    Monitor(#[from] crate::monitor::error::MonitorError),
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Failed open registry: {0:?}")]
    Open(WIN32_ERROR),
    #[error("Failed query registry: {0:?}")]
    Query(WIN32_ERROR),
    #[error("Failed set registry: {0:?}")]
    Set(WIN32_ERROR),
    #[error("Failed close registry: {0:?}")]
    Close(WIN32_ERROR),
    #[error("Failed delete registry: {0:?}")]
    Delete(WIN32_ERROR),
}
