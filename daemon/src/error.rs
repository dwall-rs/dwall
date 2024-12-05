use windows::Win32::Foundation::WIN32_ERROR;

pub type DwallResult<T> = std::result::Result<T, DwallError>;

#[derive(Debug, thiserror::Error)]
pub enum DwallError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    #[error(transparent)]
    Theme(#[from] crate::theme::ThemeError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    NulError(#[from] std::ffi::NulError),
}

// impl Serialize for DwallError {
//     fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(self.to_string().as_ref())
//     }
// }

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
