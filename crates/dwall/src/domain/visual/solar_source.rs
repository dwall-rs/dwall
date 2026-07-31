//! Solar position source for caching and providing current solar coordinates

use std::time::Duration;

use time::UtcDateTime;

use crate::{
    DwallResult,
    config::PositionSource,
    domain::{
        geography::{Position, provider::GeographicPositionProvider},
        time::solar_calculator::SolarPosition,
    },
    utils::cache::get_cache,
};

/// Provides current solar position with caching support
pub(crate) struct SolarPositionSource<'a> {
    position_provider: GeographicPositionProvider<'a>,
}

impl<'a> SolarPositionSource<'a> {
    /// Creates a new SolarPositionSource with the given position source configuration
    pub(crate) fn new(position_source: &'a PositionSource) -> Self {
        Self {
            position_provider: GeographicPositionProvider::new(position_source),
        }
    }

    /// Returns the current solar position, cached for 60 seconds
    pub(crate) fn get_current_solar_position(&self) -> DwallResult<SolarPosition> {
        let cache = get_cache();

        if let Some(cached_position) = cache.get::<SolarPosition>() {
            debug!("Using cached solar position");
            return Ok(cached_position);
        }

        let geographic_position = self.position_provider.get_current_position()?;
        let utc_time = UtcDateTime::now();
        let solar_position = SolarPosition::new(&geographic_position, &utc_time);

        cache.set(solar_position.clone(), Duration::from_secs(60));

        Ok(solar_position)
    }

    /// Returns the current geographic position
    pub(crate) fn get_current_position(&self) -> DwallResult<Position> {
        self.position_provider.get_current_position()
    }
}
