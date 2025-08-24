//! Position manager module

use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use windows::Devices::Geolocation::{Geolocator, PositionAccuracy};

use crate::{
    config::CoordinateSource,
    error::{DwallError, DwallResult},
};

use super::{
    geolocation_access::check_location_permission, position_types::Position,
    windows_api_helper::handle_windows_error,
};

/// Retrieves the current geographical position using Windows Geolocator API
///
/// This function initializes a Geolocator with high accuracy settings and
/// retrieves the current geographical coordinates.
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
    let result = Position::new_unchecked(position.Latitude, position.Longitude);
    debug!("Position struct created successfully: {}", result);

    info!(
        latitude = result.latitude(),
        longitude = result.longitude(),
        "Current geoposition"
    );
    Ok(result)
}

pub(crate) struct PositionManager {
    coordinate_source: CoordinateSource,
    fixed_position: Mutex<Option<(Position, Instant)>>,
    cache_duration: Duration,
    timeout: Duration,
}

impl PositionManager {
    pub(crate) fn new(coordinate_source: CoordinateSource) -> Self {
        Self {
            coordinate_source,
            fixed_position: Mutex::new(None),
            cache_duration: Duration::from_secs(60 * 10),
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
        let mut position = self.fixed_position.lock().await;

        match position.as_ref() {
            Some((pos, timestamp)) if timestamp.elapsed() < self.cache_duration => {
                debug!(
                    position = ?pos,
                    age_secs = timestamp.elapsed().as_secs(),
                    "Using cached position data"
                );
                Ok(pos.clone())
            }
            _ => {
                debug!("Cache expired or empty, fetching new position data");
                let new_pos = tokio::time::timeout(self.timeout, get_geo_position())
                    .await
                    .map_err(|_| {
                        error!("Position retrieval timed out after {:?}", self.timeout);
                        DwallError::Timeout("Get position".to_string())
                    })??;

                *position = Some((new_pos.clone(), Instant::now()));
                debug!(position = ?new_pos, "Updated position cache");
                Ok(new_pos)
            }
        }
    }

    /// Retrieves a position from manual coordinates
    fn get_manual_position(&self, latitude: f64, longitude: f64) -> DwallResult<Position> {
        debug!(
            latitude = latitude,
            longitude = longitude,
            "Using manual coordinates"
        );
        Position::new(latitude, longitude)
    }

    /// Retrieves the current position based on the configured coordinate source
    ///
    /// This method will either:
    /// - Use the manual coordinates if configured
    /// - Get fresh coordinates from the geolocation API if automatic update is enabled
    /// - Use cached coordinates if available and not expired, otherwise fetch new ones
    pub(crate) async fn get_current_position(&self) -> DwallResult<Position> {
        match &self.coordinate_source {
            CoordinateSource::Automatic {
                update_on_each_calculation,
            } => {
                if *update_on_each_calculation {
                    self.get_fresh_position().await
                } else {
                    self.get_cached_position().await
                }
            }
            CoordinateSource::Manual {
                latitude,
                longitude,
            } => self.get_manual_position(*latitude, *longitude),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_position_manager_manual_coordinates() {
        let manager = PositionManager::new(CoordinateSource::Manual {
            latitude: 45.0,
            longitude: 90.0,
        });

        let pos = manager.get_current_position().await.unwrap();
        assert_eq!(pos.latitude(), 45.0);
        assert_eq!(pos.longitude(), 90.0);
    }
}
