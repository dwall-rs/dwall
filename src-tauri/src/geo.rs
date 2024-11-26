use windows::Devices::Geolocation::{Geolocator, PositionAccuracy};

use crate::error::DwallResult;

pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
}

pub fn get_geo_postion() -> DwallResult<Position> {
    let geolocator = Geolocator::new()?;
    geolocator.SetDesiredAccuracy(PositionAccuracy::High)?;

    let geoposition = geolocator.GetGeopositionAsync()?.get()?;

    let coordinate = geoposition.Coordinate()?;
    let point = coordinate.Point()?;
    let position = point.Position()?;

    Ok(Position {
        latitude: position.Latitude,
        longitude: position.Longitude,
    })
}
