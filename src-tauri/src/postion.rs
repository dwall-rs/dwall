use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator};

use crate::error::DwallSettingsResult;

#[derive(Debug, thiserror::Error)]
pub enum GeolocationAccessError {
    #[error("Geolocation permission was denied by the user")]
    Denied,
    #[error("Geolocation permission status is unspecified")]
    Unspecified,
}

#[tauri::command]
pub fn request_location_permission() -> DwallSettingsResult<()> {
    trace!("Requesting geolocation access permission...");

    let access_status = Geolocator::RequestAccessAsync()?.get()?;

    match access_status {
        GeolocationAccessStatus::Allowed => {
            debug!("Geolocation permission granted.");
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
