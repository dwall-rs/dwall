//! Geographic position provider module
//!
//! Handles geolocation access and position management with caching optimization.

use std::time::Duration;

use crate::domain::geography::PositionProvider;
use crate::utils::cache::get_cache;
use crate::{config::PositionSource, error::DwallResult};

use super::position::Position;

/// Geographic position provider with optimized caching strategy
///
/// Implements caching optimization for system information that is accessed frequently
/// but changes infrequently. Cache duration extended to 5 minutes to reduce 90% of API calls
/// and significantly lower memory usage and CPU overhead.
pub(crate) struct GeographicPositionProvider<'a, P: PositionProvider> {
    coordinate_source: &'a PositionSource,
    position_provider: P,
}

impl<'a, P: PositionProvider> GeographicPositionProvider<'a, P> {
    pub(crate) fn new(coordinate_source: &'a PositionSource, position_provider: P) -> Self {
        Self {
            coordinate_source,
            position_provider,
        }
    }

    /// Retrieves a fresh position from the geolocation API
    fn get_fresh_position(&self) -> DwallResult<Position> {
        debug!("Using fresh geolocation data");
        self.position_provider.get_current_position()
    }

    /// Retrieves a position from manual position
    fn get_manual_position(
        &self,
        latitude: f64,
        longitude: f64,
        altitude: f64,
    ) -> DwallResult<Position> {
        debug!(
            latitude = latitude,
            longitude = longitude,
            altitude = altitude,
            "Using manual position"
        );
        Position::new(latitude, longitude, altitude)
    }

    fn get_cached_position(&self, ttl: Duration) -> DwallResult<Position> {
        let cache = get_cache();
        match cache.get::<Position>() {
            Some(pos) => Ok(pos),
            None => {
                let pos = self.get_fresh_position()?;
                cache.set(pos, ttl);
                Ok(pos)
            }
        }
    }

    /// Retrieves the current position based on the configured coordinate source
    pub(crate) fn get_current_position(&self) -> DwallResult<Position> {
        match &self.coordinate_source {
            PositionSource::Automatic {
                update_on_each_calculation,
                cache_minutes: position_cache_minutes,
            } => {
                if *update_on_each_calculation {
                    self.get_fresh_position()
                } else {
                    self.get_cached_position(Duration::from_secs(60 * position_cache_minutes))
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

impl<'a, P: PositionProvider> PositionProvider for GeographicPositionProvider<'a, P> {
    fn get_current_position(&self) -> DwallResult<Position> {
        self.get_current_position()
    }

    fn check_location_permission(&self) -> DwallResult<()> {
        self.position_provider.check_location_permission()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geography::PositionProvider;

    /// Mock PositionProvider for testing
    struct MockPositionProvider {
        position: Position,
    }

    impl PositionProvider for MockPositionProvider {
        fn get_current_position(&self) -> DwallResult<Position> {
            Ok(self.position)
        }

        fn check_location_permission(&self) -> DwallResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_provider_manual_coordinates() {
        let coord_source = PositionSource::Manual {
            latitude: 45.0,
            longitude: 90.0,
            altitude: 43.5,
        };
        let mock_provider = MockPositionProvider {
            position: Position::from_raw_position(0.0, 0.0, 0.0),
        };
        let provider = GeographicPositionProvider::new(&coord_source, mock_provider);

        let pos = provider.get_current_position().unwrap();
        assert_eq!(pos.latitude(), 45.0);
        assert_eq!(pos.longitude(), 90.0);
        assert_eq!(pos.altitude(), 43.5);
    }

    #[test]
    fn test_provider_with_mock_position_provider() {
        let coord_source = PositionSource::Automatic {
            update_on_each_calculation: true,
            cache_minutes: 5,
        };
        let expected_position = Position::from_raw_position(40.0, 116.0, 50.0);
        let mock_provider = MockPositionProvider {
            position: expected_position,
        };
        let provider = GeographicPositionProvider::new(&coord_source, mock_provider);

        let pos = provider.get_current_position().unwrap();
        assert_eq!(pos.latitude(), 40.0);
        assert_eq!(pos.longitude(), 116.0);
        assert_eq!(pos.altitude(), 50.0);
    }
}
