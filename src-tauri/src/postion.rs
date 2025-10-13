use dwall::domain::geography::check_location_permission;

use crate::error::DwallSettingsResult;

#[tauri::command]
pub async fn request_location_permission() -> DwallSettingsResult<()> {
    check_location_permission().await.map_err(Into::into)
}
