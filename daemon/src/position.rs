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

pub fn get_geo_position() -> DwallResult<Position> {
    trace!("Initializing Geolocator...");
    let geolocator = Geolocator::new().map_err(|e| {
        error!("Failed to initialize Geolocator: {:?}", e);
        e
    })?;
    debug!("Geolocator initialized successfully.");

    trace!("Setting desired accuracy to High...");
    geolocator
        .SetDesiredAccuracy(PositionAccuracy::High)
        .map_err(|e| {
            error!("Failed to set desired accuracy to High: {:?}", e);
            e
        })?;
    debug!("Desired accuracy set to High successfully.");

    trace!("Getting geoposition asynchronously...");
    let geoposition = geolocator.GetGeopositionAsync()?.get().map_err(|e| {
        error!("Failed to retrieve geoposition: {:?}", e);
        e
    })?;
    debug!("Geoposition retrieved successfully.");

    trace!("Extracting coordinate from geoposition...");
    let coordinate = geoposition.Coordinate().map_err(|e| {
        error!("Failed to extract coordinate from geoposition: {:?}", e);
        e
    })?;
    debug!("Coordinate extracted successfully.");

    trace!("Extracting point from coordinate...");
    let point = coordinate.Point().map_err(|e| {
        error!("Failed to extract point from coordinate: {:?}", e);
        e
    })?;
    debug!("Point extracted successfully.");

    trace!("Extracting position from point...");
    let position = point.Position().map_err(|e| {
        error!("Failed to extract position from point: {:?}", e);
        e
    })?;
    debug!("Position extracted successfully.");

    trace!("Creating Position struct with latitude and longitude...");
    let result = Position {
        latitude: position.Latitude,
        longitude: position.Longitude,
    };
    debug!("Position struct created successfully.");

    info!("Current geoposition: {:?}", result);
    Ok(result)
}

pub struct PositionManager {
    coordinate_source: CoordinateSource,
    fixed_position: Mutex<Option<Position>>,
}

impl PositionManager {
    pub fn new(coordinate_source: CoordinateSource) -> Self {
        Self {
            coordinate_source,
            fixed_position: Mutex::new(None),
        }
    }

    pub async fn get_current_position(&self) -> DwallResult<Position> {
        match &self.coordinate_source {
            CoordinateSource::Automatic {
                update_on_each_calculation,
            } => {
                if *update_on_each_calculation {
                    get_geo_position()
                } else {
                    let mut position = self.fixed_position.lock().await;
                    Ok(position
                        .get_or_insert_with(|| get_geo_position().unwrap())
                        .clone())
                }
            }
            CoordinateSource::Manual {
                latitude,
                longitude,
            } => Ok(Position::new(*latitude, *longitude)),
        }
    }
}
