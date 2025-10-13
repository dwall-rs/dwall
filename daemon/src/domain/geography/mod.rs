pub mod coordinates;
pub mod provider;

// Re-export commonly used types
pub use coordinates::{Coordinate, CoordinateError, GeolocationAccessError};
pub use provider::check_location_permission;
