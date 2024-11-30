use windows::Devices::Geolocation::{Geolocator, PositionAccuracy};

use crate::error::DwallResult;

#[derive(Debug)]
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
