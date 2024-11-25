use windows::{
    Devices::Geolocation::{Geolocator, PositionAccuracy},
    Foundation::IAsyncOperation,
};

pub async fn get_geo_postion() -> windows::Result<()> {
    let geolcator = Geolocator::new()?;
    geolocator.SetDesiredAccuracy(PositionAccuracy::High)?;

    let geoposition = geolocator.GetGeopositionAsync()?.await?;

    let coordinate = geoposition.Coordinate()?;
    let point = coordinate.Point()?;
    let position = point.Position()?;

    println!(
        "Longitude: {}, Latitude: {}",
        position.Longitude, position.Latitude
    );

    Ok(())
}
