use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator};

use crate::error::{DwallError, DwallResult};

use super::windows_api_helper::handle_windows_error;

#[derive(Debug, thiserror::Error)]
pub enum GeolocationAccessError {
    #[error("Geolocation permission was denied by the user")]
    Denied,
    #[error("Geolocation permission status is unspecified")]
    Unspecified,
}

/// Checks if the application has permission to access location
///
/// Returns Ok(()) if permission is granted, or an error if denied or unspecified
pub async fn check_location_permission() -> DwallResult<()> {
    let access_status = handle_windows_error(
        "Requesting geolocation access permission",
        Geolocator::RequestAccessAsync,
    )
    .await?
    .get()
    .map_err(|e| {
        error!(error = ?e, "Failed to get access status");
        DwallError::Windows(e)
    })?;

    match access_status {
        GeolocationAccessStatus::Allowed => {
            debug!("Geolocation permission granted");
            Ok(())
        }
        GeolocationAccessStatus::Denied => {
            error!("{}", GeolocationAccessError::Denied);
            Err(GeolocationAccessError::Denied.into())
        }
        GeolocationAccessStatus::Unspecified => {
            error!("{}", GeolocationAccessError::Unspecified);
            Err(GeolocationAccessError::Unspecified.into())
        }
        _ => unreachable!(),
    }
}
