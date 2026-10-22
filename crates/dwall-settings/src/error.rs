use dwall::RegistryError;
use serde::{Serialize, Serializer};

use crate::filesystem::DirectoryMoveError;

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
    #[error("Failed to spawn daemon: {0}")]
    Daemon(String),
    /// The user cancelled the download.
    ///
    /// A variant of its own so the retry loop can tell "the user asked us to stop"
    /// from "the transfer failed", which used to be a string comparison. The message
    /// is what the renderer matches on (`isCancellation`), so it stays as it was.
    #[error("Download cancelled")]
    Cancelled,
    /// The transfer would have written more than the caller allowed.
    ///
    /// A variant of its own for the same reason as [`Self::Cancelled`]: the retry loop
    /// has to tell it from a flaky transfer. The server answered the same way once and
    /// will answer the same way again, so retrying only writes the same bytes again.
    #[error("Download exceeds the {limit} byte limit")]
    DownloadTooLarge { limit: u64 },
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
