use dwall::RegistryError;
use serde::{Serialize, Serializer};

use crate::infrastructure::{filesystem::DirectoryMoveError, network::download::DownloadError};

pub type DwallSettingsResult<T, E = DwallSettingsError> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum DwallSettingsError {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Update(#[from] tauri_plugin_updater::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Dwall(#[from] dwall::error::DwallError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
    #[error(transparent)]
    Download(#[from] DownloadError),
    #[error("Failed to spawn daemon: {0}")]
    Daemon(String),
    #[error(transparent)]
    Logger(#[from] log::SetLoggerError),
    #[error(transparent)]
    DirectoryMove(#[from] DirectoryMoveError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    TomlDeserialize(#[from] toml::de::Error),
    #[error("{0}")]
    Other(String),
}

impl Serialize for DwallSettingsError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
