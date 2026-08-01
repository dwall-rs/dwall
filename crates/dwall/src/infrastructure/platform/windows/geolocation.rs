//! Geographic position provider module
//!
//! Handles geolocation access and position management with caching optimization.

use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator, PositionAccuracy};

use crate::domain::geography::{GeolocationAccessError, PositionProvider};
use crate::error::{DwallError, DwallResult};

use crate::domain::geography::position::Position;

/// Helper function to handle Windows API errors with consistent logging
fn handle_windows_error<T, F>(operation: &str, f: F) -> DwallResult<T>
where
    F: FnOnce() -> windows::core::Result<T>,
{
    trace!("{}", operation);
    match f() {
        Ok(result) => {
            debug!("{} completed successfully", operation);
            Ok(result)
        }
        Err(e) => {
            error!(error = e, "{} failed", operation);
            Err(DwallError::Windows(e))
        }
    }
}

pub struct Positioner;

impl Positioner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Positioner {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionProvider for Positioner {
    fn get_current_position(&self) -> DwallResult<Position> {
        // First check if we have permission to access location
        self.check_location_permission()?;

        // Initialize geolocator
        let geolocator = handle_windows_error("Initializing Geolocator", Geolocator::new)?;

        // Set accuracy to high
        handle_windows_error("Setting desired accuracy to High", || {
            geolocator.SetDesiredAccuracy(PositionAccuracy::High)
        })?;

        // Get geoposition
        let geoposition = handle_windows_error("Getting geoposition asynchronously", || {
            geolocator.GetGeopositionAsync()
        })?
        .get()
        .inspect_err(|e| {
            error!(error = e, "Failed to retrieve geoposition");
        })?;

        // Extract coordinate
        let coordinate = handle_windows_error("Extracting coordinate from geoposition", || {
            geoposition.Coordinate()
        })?;

        // Extract point
        let point =
            handle_windows_error("Extracting point from coordinate", || coordinate.Point())?;

        // Extract position
        let position = handle_windows_error("Extracting position from point", || point.Position())?;

        // Create Position struct
        trace!("Creating Position struct with latitude and longitude...");
        let position =
            Position::from_raw_position(position.Latitude, position.Longitude, position.Altitude);
        if position.altitude() == 0. {
            warn!(
                "An altitude of 0 may cause the time for switching between light and dark modes to shift earlier or later by a few minutes to an hour. This is likely because your device lacks a barometric pressure sensor. This is not an error, but an expected outcome."
            );
        }

        info!(
            latitude = position.latitude(),
            longitude = position.longitude(),
            altitude = position.altitude(),
            "Current geoposition"
        );
        Ok(position)
    }

    fn check_location_permission(&self) -> DwallResult<()> {
        let access_status = handle_windows_error(
            "Requesting geolocation access permission",
            Geolocator::RequestAccessAsync,
        )?
        .get()
        .inspect_err(|e| {
            error!(error = e, "Failed to get access status");
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
}
