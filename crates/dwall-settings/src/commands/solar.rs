//! Solar position command

use dwall::{
    GeographicPositionProvider, SolarPosition, config::PositionSource, platform::Positioner,
};
use time::UtcDateTime;

use crate::error::DwallSettingsResult;

#[tauri::command]
pub fn get_solar_position(
    position_source: PositionSource,
    timestamp: u64,
) -> DwallSettingsResult<SolarPosition> {
    let position_provider = GeographicPositionProvider::new(&position_source, Positioner::new());
    let position = position_provider.get_current_position()?;
    let date_time = UtcDateTime::from_timestamp(timestamp);
    Ok(SolarPosition::new(&position, &date_time))
}
