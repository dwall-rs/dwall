use crate::error::DwallSettingsResult;

#[tauri::command]
pub async fn request_location_permission() -> DwallSettingsResult<()> {
    dwall::position::check_location_permission()
        .await
        .map_err(Into::into)
}
