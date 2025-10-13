use std::fmt;

use crate::error::DwallResult;

/// Geographic position with latitude and longitude coordinates
///
/// This struct is optimized for performance with Copy trait and
/// uses repr(C) for predictable memory layout.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Coordinate {
    latitude: f64,
    longitude: f64,
}

impl Coordinate {
    /// Creates a new Coordinate with the given latitude and longitude
    ///
    /// # Arguments
    /// * `latitude` - The latitude in degrees, must be between -90 and 90
    /// * `longitude` - The longitude in degrees, must be between -180 and 180
    ///
    /// # Returns
    /// A new Coordinate instance if the coordinates are valid, or PositionError if they are out of range
    pub fn new(latitude: f64, longitude: f64) -> DwallResult<Self> {
        if !Self::is_valid_latitude(latitude) {
            return Err(CoordinateError::InvalidLatitude(latitude).into());
        }
        if !Self::is_valid_longitude(longitude) {
            return Err(CoordinateError::InvalidLongitude(longitude).into());
        }

        Ok(Coordinate {
            latitude,
            longitude,
        })
    }

    /// Creates a new Coordinate without validation
    ///
    /// # Safety
    /// This method bypasses coordinate validation. Only use when coordinates
    /// are guaranteed to be valid (e.g., from trusted sources).
    ///
    /// # Arguments
    /// * `latitude` - Latitude in degrees (should be between -90 and 90)
    /// * `longitude` - Longitude in degrees (should be between -180 and 180)
    pub(crate) fn from_raw_coordinates(latitude: f64, longitude: f64) -> Self {
        Coordinate {
            latitude,
            longitude,
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
}

impl Default for Coordinate {
    fn default() -> Self {
        // Default to coordinates (0, 0) - null island
        Self {
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Coordinate(lat: {}, lng: {})",
            self.latitude, self.longitude
        )
    }
}

/// Error type for position-related operations
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

    #[tokio::test]
    async fn test_position_new() {
        let pos = Coordinate::new(45.0, 90.0).unwrap();
        assert_eq!(pos.latitude(), 45.0);
        assert_eq!(pos.longitude(), 90.0);
    }

    #[tokio::test]
    async fn test_position_new_invalid() {
        assert!(Coordinate::new(91.0, 90.0).is_err());
        assert!(Coordinate::new(45.0, 181.0).is_err());
        assert!(Coordinate::new(-91.0, 0.0).is_err());
        assert!(Coordinate::new(0.0, -181.0).is_err());
    }

    #[tokio::test]
    async fn test_position_validation() {
        assert!(Coordinate::is_valid_latitude(90.0));
        assert!(Coordinate::is_valid_latitude(-90.0));
        assert!(Coordinate::is_valid_latitude(0.0));
        assert!(!Coordinate::is_valid_latitude(90.1));
        assert!(!Coordinate::is_valid_latitude(-90.1));

        assert!(Coordinate::is_valid_longitude(180.0));
        assert!(Coordinate::is_valid_longitude(-180.0));
        assert!(Coordinate::is_valid_longitude(0.0));
        assert!(!Coordinate::is_valid_longitude(180.1));
        assert!(!Coordinate::is_valid_longitude(-180.1));
    }

    #[tokio::test]
    async fn test_position_default() {
        let pos = Coordinate::default();
        assert_eq!(pos.latitude(), 0.0);
        assert_eq!(pos.longitude(), 0.0);
    }

    #[tokio::test]
    async fn test_position_display() {
        let pos = Coordinate::from_raw_coordinates(45.0, 90.0);
        assert_eq!(format!("{pos}"), "Coordinate(lat: 45, lng: 90)");
    }
}
