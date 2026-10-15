//! Solar position commands

use dwall::{
    GeographicPositionProvider, SolarPosition, config::PositionSource, platform::Positioner,
};
use time::UtcDateTime;

use crate::error::DwallSettingsResult;

/// Sample interval for the diurnal sun path (10 minutes).
const PATH_SAMPLE_SECONDS: u64 = 10 * 60;
const DAY_SECONDS: u64 = 24 * 60 * 60;

/// Runs off the main thread: reading the current position blocks on the WinRT
/// geolocation request (and may show the system permission prompt).
#[tauri::command]
pub async fn get_solar_position(
    position_source: PositionSource,
    timestamp: u64,
) -> DwallSettingsResult<SolarPosition> {
    tauri::async_runtime::spawn_blocking(move || -> DwallSettingsResult<SolarPosition> {
        let position_provider =
            GeographicPositionProvider::new(&position_source, Positioner::new());
        let position = position_provider.get_current_position()?;
        let date_time = UtcDateTime::from_timestamp(timestamp);
        Ok(SolarPosition::new(&position, &date_time))
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}

/// Apparent solar positions sampled over a full day, centred on `timestamp`.
///
/// Used to draw the diurnal sun-trajectory preview: unlike a theme's
/// `solar.json` (which holds the wallpaper keyframes), this is the real path
/// for the observer's location and date, so it always forms a smooth ellipse.
#[tauri::command]
pub async fn get_solar_path(
    position_source: PositionSource,
    timestamp: u64,
) -> DwallSettingsResult<Vec<SolarPosition>> {
    tauri::async_runtime::spawn_blocking(move || -> DwallSettingsResult<Vec<SolarPosition>> {
        let position_provider =
            GeographicPositionProvider::new(&position_source, Positioner::new());
        let position = position_provider.get_current_position()?;

        let start = timestamp.saturating_sub(DAY_SECONDS / 2);
        let samples = (0..=DAY_SECONDS / PATH_SAMPLE_SECONDS)
            .map(|i| {
                let date_time = UtcDateTime::from_timestamp(start + i * PATH_SAMPLE_SECONDS);
                SolarPosition::new(&position, &date_time)
            })
            .collect();

        Ok(samples)
    })
    .await
    .map_err(|e| crate::error::DwallSettingsError::Other(e.to_string()))?
}
