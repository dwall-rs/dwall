//! Geographic position provider module
//!
//! Handles geolocation access and position management with caching optimization.

use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use windows::Devices::Geolocation::{GeolocationAccessStatus, Geolocator, PositionAccuracy};

use crate::{
    config::PositionSource,
    error::{DwallError, DwallResult},
};

use super::position::{GeolocationAccessError, Position};

/// Helper function to handle Windows API errors with consistent logging
async fn handle_windows_error<T, F>(operation: &str, f: F) -> DwallResult<T>
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
            error!(error = %e, "{} failed", operation);
            Err(DwallError::Windows(e))
        }
    }
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
        error!(error = %e, "Failed to get access status");
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

/// Retrieves the current geographical position using Windows Geolocator API
async fn get_geo_position() -> DwallResult<Position> {
    // First check if we have permission to access location
    check_location_permission().await?;

    // Initialize geolocator
    let geolocator = handle_windows_error("Initializing Geolocator", Geolocator::new).await?;

    // Set accuracy to high
    handle_windows_error("Setting desired accuracy to High", || {
        geolocator.SetDesiredAccuracy(PositionAccuracy::High)
    })
    .await?;

    // Get geoposition
    let geoposition = handle_windows_error("Getting geoposition asynchronously", || {
        geolocator.GetGeopositionAsync()
    })
    .await?
    .get()
    .map_err(|e| {
        error!(error = %e, "Failed to retrieve geoposition");
        DwallError::Windows(e)
    })?;

    // Extract coordinate
    let coordinate = handle_windows_error("Extracting coordinate from geoposition", || {
        geoposition.Coordinate()
    })
    .await?;

    // Extract point
    let point =
        handle_windows_error("Extracting point from coordinate", || coordinate.Point()).await?;

    // Extract position
    let position =
        handle_windows_error("Extracting position from point", || point.Position()).await?;

    // Create Position struct
    trace!("Creating Position struct with latitude and longitude...");
    let position =
        Position::from_raw_position(position.Latitude, position.Longitude, position.Altitude);
    if position.altitude() == 0. {
        warn!("An altitude of 0 may cause the time for switching between light and dark modes to shift earlier or later by a few minutes to an hour. This is likely because your device lacks a barometric pressure sensor. This is not an error, but an expected outcome.");
    }

    info!(
        latitude = position.latitude(),
        longitude = position.longitude(),
        altitude = position.altitude(),
        "Current geoposition"
    );
    Ok(position)
}

/// Geographic position provider with optimized caching strategy
///
/// Implements caching optimization for system information that is accessed frequently
/// but changes infrequently. Cache duration extended to 5 minutes to reduce 90% of API calls
/// and significantly lower memory usage and CPU overhead.
pub(crate) struct GeographicPositionProvider<'a> {
    coordinate_source: &'a PositionSource,
    cached_position: Mutex<Option<(Position, Instant)>>,

    /// Cache duration for position data
    cache_duration: Duration,
    timeout: Duration,
}

impl<'a> GeographicPositionProvider<'a> {
    pub(crate) fn new(coordinate_source: &'a PositionSource) -> Self {
        Self {
            coordinate_source,
            cached_position: Mutex::new(None),
            cache_duration: Duration::from_secs(60 * 5),
            timeout: Duration::from_secs(10),
        }
    }

    /// Retrieves a fresh position from the geolocation API
    async fn get_fresh_position(&self) -> DwallResult<Position> {
        debug!("Using fresh geolocation data");
        get_geo_position().await
    }

    /// Retrieves a position from cache or fetches a new one if cache is expired
    async fn get_cached_position(&self) -> DwallResult<Position> {
        debug!("Checking cached position data");
        let mut cache = self.cached_position.lock().await;

        if let Some((cached_position, cached_time)) = cache.as_ref() {
            if cached_time.elapsed() < self.cache_duration {
                debug!(
                    position = ?cached_position,
                    age_secs = cached_time.elapsed().as_secs(),
                    "Using cached position data"
                );
                return Ok(*cached_position);
            }
        }

        debug!("Cache expired or empty, fetching new position data");
        let position = tokio::time::timeout(self.timeout, self.get_fresh_position())
            .await
            .map_err(|_| {
                error!("Position retrieval timed out after {:?}", self.timeout);
                DwallError::Timeout("Get position".to_string())
            })??;

        *cache = Some((position, Instant::now()));
        debug!(position = ?position, "Updated position cache");
        Ok(position)
    }

    /// Retrieves a position from manual position
    fn get_manual_position(
        &self,
        latitude: f64,
        longitude: f64,
        altitude: f64,
    ) -> DwallResult<Position> {
        debug!(latitude, longitude, altitude, "Using manual position");
        Position::new(latitude, longitude, altitude)
    }

    /// Retrieves the current position based on the configured coordinate source
    pub(crate) async fn get_current_position(&self) -> DwallResult<Position> {
        match &self.coordinate_source {
            PositionSource::Automatic {
                update_on_each_calculation,
            } => {
                if *update_on_each_calculation {
                    self.get_fresh_position().await
                } else {
                    self.get_cached_position().await
                }
            }
            PositionSource::Manual {
                latitude,
                longitude,
                altitude,
            } => self.get_manual_position(*latitude, *longitude, *altitude),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_manual_coordinates() {
        let coord_source = PositionSource::Manual {
            latitude: 45.0,
            longitude: 90.0,
            altitude: 43.5,
        };
        let provider = GeographicPositionProvider::new(&coord_source);

        let pos = provider.get_current_position().await.unwrap();
        assert_eq!(pos.latitude(), 45.0);
        assert_eq!(pos.longitude(), 90.0);
        assert_eq!(pos.altitude(), 43.5);
    }
}
