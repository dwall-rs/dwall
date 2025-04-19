use std::fmt;

use crate::error::DwallResult;

#[derive(Debug, Clone)]
pub struct Position {
    latitude: f64,
    longitude: f64,
}

impl Position {
    /// Creates a new Position with the given latitude and longitude
    ///
    /// # Arguments
    /// * `latitude` - The latitude in degrees, must be between -90 and 90
    /// * `longitude` - The longitude in degrees, must be between -180 and 180
    ///
    /// # Returns
    /// A new Position instance if the coordinates are valid, or PositionError if they are out of range
    pub fn new(latitude: f64, longitude: f64) -> DwallResult<Self> {
        if !Self::is_valid_latitude(latitude) {
            return Err(PositionError::InvalidLatitude(latitude).into());
        }
        if !Self::is_valid_longitude(longitude) {
            return Err(PositionError::InvalidLongitude(longitude).into());
        }

        Ok(Position {
            latitude,
            longitude,
        })
    }

    /// Creates a new Position without validation
    ///
    /// # Warning
    /// This method should only be used when the coordinates are already validated
    pub(crate) fn new_unchecked(latitude: f64, longitude: f64) -> Self {
        Position {
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

impl Default for Position {
    fn default() -> Self {
        // Default to coordinates (0, 0) - null island
        Self {
            latitude: 0.0,
            longitude: 0.0,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Position(lat: {}, lng: {})",
            self.latitude, self.longitude
        )
    }
}

/// Error type for position-related operations
#[derive(Debug, thiserror::Error)]
pub enum PositionError {
    #[error("Invalid latitude: {0}. Must be between -90 and 90 degrees")]
    InvalidLatitude(f64),

    #[error("Invalid longitude: {0}. Must be between -180 and 180 degrees")]
    InvalidLongitude(f64),

    #[error("Invalid coordinates: latitude {0}, longitude {1}")]
    InvalidCoordinates(f64, f64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_position_new() {
        let pos = Position::new(45.0, 90.0).unwrap();
        assert_eq!(pos.latitude(), 45.0);
        assert_eq!(pos.longitude(), 90.0);
    }

    #[test]
    async fn test_position_new_invalid() {
        assert!(Position::new(91.0, 90.0).is_err());
        assert!(Position::new(45.0, 181.0).is_err());
        assert!(Position::new(-91.0, 0.0).is_err());
        assert!(Position::new(0.0, -181.0).is_err());
    }

    #[test]
    async fn test_position_validation() {
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
    async fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.latitude(), 0.0);
        assert_eq!(pos.longitude(), 0.0);
    }

    #[test]
    async fn test_position_display() {
        let pos = Position::new_unchecked(45.0, 90.0);
        assert_eq!(format!("{}", pos), "Position(lat: 45, lng: 90)");
    }
}
