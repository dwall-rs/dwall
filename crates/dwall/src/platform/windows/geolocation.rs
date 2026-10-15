//! Windows implementation of [`PositionProvider`] using the Geolocator API.

use std::time::Duration;

use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator, PositionAccuracy};
use windows::Foundation::TimeSpan;

use crate::error::{DwallError, DwallResult};
use crate::solar::{GeolocationAccessError, Position, PositionProvider};

use super::async_operation;

/// Maximum acceptable age of a cached reading before a fresh sensor read is requested.
const MAX_GEOPOSITION_AGE: Duration = Duration::from_secs(60);

/// Upper bound on how long a reading may block the caller.
///
/// The WinRT overload used below enforces this budget inside the platform call (an
/// unresponsive location service returns a timeout instead of waiting for the
/// sensor), and the wait itself is bounded by the same value as a backstop.
const GEOPOSITION_TIMEOUT: Duration = Duration::from_secs(30);

/// Upper bound on waiting for the access-status request.
///
/// `RequestAccessAsync` has no platform-side budget of its own, so this wait is the
/// only thing keeping a hung broker from blocking the daemon.
const ACCESS_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Converts a [`Duration`] into a WinRT `TimeSpan` (100-nanosecond ticks).
fn time_span(duration: Duration) -> TimeSpan {
    TimeSpan {
        Duration: (duration.as_secs() as i64) * 10_000_000,
    }
}

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

        // Get geoposition with an explicit age/timeout budget instead of the
        // no-argument overload, which can block indefinitely.
        let operation = handle_windows_error("Getting geoposition asynchronously", || {
            geolocator.GetGeopositionAsyncWithAgeAndTimeout(
                time_span(MAX_GEOPOSITION_AGE),
                time_span(GEOPOSITION_TIMEOUT),
            )
        })?;

        let geoposition =
            async_operation::wait(&operation, GEOPOSITION_TIMEOUT).inspect_err(|e| {
                error!(error = %e, "Failed to retrieve geoposition");
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

        // Coordinates are personal data and the log directory is one click away in
        // the settings UI, so they stay out of the default (info) log stream.
        debug!(
            latitude = position.latitude(),
            longitude = position.longitude(),
            altitude = position.altitude(),
            "Current geoposition"
        );
        Ok(position)
    }

    fn check_location_permission(&self) -> DwallResult<()> {
        let operation = handle_windows_error(
            "Requesting geolocation access permission",
            Geolocator::RequestAccessAsync,
        )?;

        let access_status =
            async_operation::wait(&operation, ACCESS_REQUEST_TIMEOUT).inspect_err(|e| {
                error!(error = %e, "Failed to get access status");
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
            // `GeolocationAccessStatus` is a struct with associated constants, so
            // this arm is reachable: a value this build does not know about must not
            // abort the daemon.
            other => {
                warn!(
                    status = other.0,
                    "Unexpected geolocation access status, treating it as unspecified"
                );
                Err(GeolocationAccessError::Unspecified.into())
            }
        }
    }
}
