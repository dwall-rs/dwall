use serde::{Serialize, Serializer};

pub(super) type DwallResult<T> = std::result::Result<T, DwallError>;

#[derive(Debug, thiserror::Error)]
pub(super) enum DwallError {
    #[error(transparent)]
    Update(#[from] tauri_plugin_updater::Error),
}

impl Serialize for DwallError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
