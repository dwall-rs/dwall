use tokio::sync::Mutex;
use windows::Devices::Geolocation::{Geolocator, PositionAccuracy};

use crate::{config::CoordinateSource, error::DwallResult};

#[derive(Debug, Clone)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

impl Position {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        Position {
            latitude,
            longitude,
        }
    }
}

pub async fn get_geo_position() -> DwallResult<Position> {
    trace!("Initializing Geolocator...");
    let geolocator = Geolocator::new().map_err(|e| {
        error!(error = ?e, "Failed to initialize Geolocator");
        e
    })?;
    debug!("Geolocator initialized successfully.");

    trace!("Setting desired accuracy to High...");
    geolocator
        .SetDesiredAccuracy(PositionAccuracy::High)
        .map_err(|e| {
            error!(error = ?e, "Failed to set desired accuracy to High");
            e
        })?;
    debug!("Desired accuracy set to High successfully.");

    trace!("Getting geoposition asynchronously...");
    let geoposition = geolocator.GetGeopositionAsync()?.get().map_err(|e| {
        error!(error = ?e, "Failed to retrieve geoposition");
        e
    })?;
    debug!("Geoposition retrieved successfully.");

    trace!("Extracting coordinate from geoposition...");
    let coordinate = geoposition.Coordinate().map_err(|e| {
        error!(error = ?e, "Failed to extract coordinate from geoposition");
        e
    })?;
    debug!("Coordinate extracted successfully.");

    trace!("Extracting point from coordinate...");
    let point = coordinate.Point().map_err(|e| {
        error!(error = ?e, "Failed to extract point from coordinate");
        e
    })?;
    debug!("Point extracted successfully.");

    trace!("Extracting position from point...");
    let position = point.Position().map_err(|e| {
        error!(error = ?e, "Failed to extract position from point");
        e
    })?;
    debug!("Position extracted successfully.");

    trace!("Creating Position struct with latitude and longitude...");
    let result = Position {
        latitude: position.Latitude,
        longitude: position.Longitude,
    };
    debug!("Position struct created successfully.");

    info!(
        latitude = result.latitude,
        longitude = result.longitude,
        "Current geoposition"
    );
    Ok(result)
}

pub struct PositionManager {
    coordinate_source: CoordinateSource,
    fixed_position: Mutex<Option<(Position, std::time::Instant)>>,
    cache_duration: std::time::Duration,
    timeout: std::time::Duration,
}

impl PositionManager {
    pub fn new(coordinate_source: CoordinateSource) -> Self {
        Self {
            coordinate_source,
            fixed_position: Mutex::new(None),
            cache_duration: std::time::Duration::from_secs(60 * 5),
            timeout: std::time::Duration::from_secs(10),
        }
    }

    // pub fn with_cache_duration(mut self, duration: std::time::Duration) -> Self {
    //     self.cache_duration = duration;
    //     self
    // }

    // pub fn with_timeout(mut self, duration: std::time::Duration) -> Self {
    //     self.timeout = duration;
    //     self
    // }

    pub async fn get_current_position(&self) -> DwallResult<Position> {
        match &self.coordinate_source {
            CoordinateSource::Automatic {
                update_on_each_calculation,
            } => {
                if *update_on_each_calculation {
                    get_geo_position().await
                } else {
                    let mut position = self.fixed_position.lock().await;
                    match position.as_ref() {
                        Some((pos, timestamp)) if timestamp.elapsed() < self.cache_duration => {
                            Ok(pos.clone())
                        }
                        _ => {
                            let new_pos = tokio::time::timeout(self.timeout, get_geo_position())
                                .await
                                .map_err(|_| {
                                    crate::error::DwallError::TimeoutError(
                                        "Get position".to_string(),
                                    )
                                })??;
                            *position = Some((new_pos.clone(), std::time::Instant::now()));
                            Ok(new_pos)
                        }
                    }
                }
            }
            CoordinateSource::Manual {
                latitude,
                longitude,
            } => Ok(Position::new(*latitude, *longitude)),
        }
    }
}
