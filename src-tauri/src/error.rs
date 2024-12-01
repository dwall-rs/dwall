use dwall::error::RegistryError;
use serde::{Serialize, Serializer};

pub type DwallSettingsResult<T> = std::result::Result<T, DwallSettingsError>;

#[derive(Debug, thiserror::Error)]
pub enum DwallSettingsError {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Update(#[from] tauri_plugin_updater::Error),
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error(transparent)]
    ZipExtract(#[from] zip_extract::ZipExtractError),
    #[error(transparent)]
    Dwall(#[from] dwall::DwallError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
}

impl Serialize for DwallSettingsError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
