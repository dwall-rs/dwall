use std::fmt;

use crate::error::DwallResult;

/// Geographic position with latitude, longitude and altitude
///
/// This struct is optimized for performance with Copy trait and
/// uses repr(C) for predictable memory layout.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Position {
    latitude: f64,
    longitude: f64,
    altitude: f64,
}

impl Position {
    /// Creates a new Position with the given latitude, longitude and altitude
    ///
    /// # Arguments
    /// * `latitude` - The latitude in degrees, must be between -90 and 90
    /// * `longitude` - The longitude in degrees, must be between -180 and 180
    /// * `altitude` - The altitude in meters, can be negative
    ///
    /// # Returns
    /// A new Position instance if the coordinates are valid, or CoordinateError if they are out of range
    pub fn new(latitude: f64, longitude: f64, altitude: f64) -> DwallResult<Self> {
        if !Self::is_valid_latitude(latitude) {
            return Err(CoordinateError::InvalidLatitude(latitude).into());
        }
        if !Self::is_valid_longitude(longitude) {
            return Err(CoordinateError::InvalidLongitude(longitude).into());
        }

        Ok(Position {
            latitude,
            longitude,
            altitude,
        })
    }

    /// Creates a new Position without validation
    ///
    /// # Safety
    /// This method bypasses coordinate validation. Only use when coordinates
    /// are guaranteed to be valid (e.g., from trusted sources).
    ///
    /// # Arguments
    /// * `latitude` - Latitude in degrees (should be between -90 and 90)
    /// * `longitude` - Longitude in degrees (should be between -180 and 180)
    /// * `altitude` - Altitude in meters (should be negative for underground positions)
    pub(crate) fn from_raw_position(latitude: f64, longitude: f64, altitude: f64) -> Self {
        Position {
            latitude,
            longitude,
            altitude,
        }
    }

    /// Checks if the given latitude is valid (between -90 and 90 degrees)
    pub fn is_valid_latitude(latitude: f64) -> bool {
        (-90.0..=90.0).contains(&latitude)
    }

    /// Checks if the given longitude is valid (between -180 and 180 degrees)
    pub fn is_valid_longitude(longitude: f64) -> bool {
        (-180.0..=180.0).contains(&longitude)
    }

    pub fn latitude(&self) -> f64 {
        self.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.longitude
    }

    pub fn altitude(&self) -> f64 {
        self.altitude
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Position(lat: {}, lng: {}, alt: {})",
            self.latitude, self.longitude, self.altitude
        )
    }
}

/// Error type for coordinate-related operations
#[derive(Debug, thiserror::Error)]
pub enum CoordinateError {
    #[error("Invalid latitude: {0}. Must be between -90 and 90 degrees")]
    InvalidLatitude(f64),

    #[error("Invalid longitude: {0}. Must be between -180 and 180 degrees")]
    InvalidLongitude(f64),

    #[error("Invalid coordinates: latitude {0}, longitude {1}")]
    InvalidCoordinates(f64, f64),
}

/// Error type for geolocation access operations
#[derive(Debug, thiserror::Error)]
pub enum GeolocationAccessError {
    #[error("Geolocation permission was denied by the user")]
    Denied,
    #[error("Geolocation permission status is unspecified")]
    Unspecified,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(45.0, 90.0, 43.5).unwrap();
        assert_eq!(pos.latitude(), 45.0);
        assert_eq!(pos.longitude(), 90.0);
        assert_eq!(pos.altitude(), 43.5);
    }

    #[test]
    fn test_position_new_invalid() {
        assert!(Position::new(91.0, 90.0, 43.5).is_err());
        assert!(Position::new(45.0, 181.0, 43.5).is_err());
        assert!(Position::new(-91.0, 0.0, 43.5).is_err());
        assert!(Position::new(0.0, -181.0, 43.5).is_err());
    }

    #[test]
    fn test_position_validation() {
        assert!(Position::is_valid_latitude(90.0));
        assert!(Position::is_valid_latitude(-90.0));
        assert!(Position::is_valid_latitude(0.0));
        assert!(!Position::is_valid_latitude(90.1));
        assert!(!Position::is_valid_latitude(-90.1));

        assert!(Position::is_valid_longitude(180.0));
        assert!(Position::is_valid_longitude(-180.0));
        assert!(Position::is_valid_longitude(0.0));
        assert!(!Position::is_valid_longitude(180.1));
        assert!(!Position::is_valid_longitude(-180.1));
    }

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.latitude(), 0.0);
        assert_eq!(pos.longitude(), 0.0);
    }

    #[test]
    fn test_position_display() {
        let pos = Position::from_raw_position(45.0, 90.0, 43.5);
        assert_eq!(format!("{pos}"), "Position(lat: 45, lng: 90, alt: 43.5)");
    }
}
