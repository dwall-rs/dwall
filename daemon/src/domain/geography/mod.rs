pub mod position;
pub mod provider;

// Re-export commonly used types
pub use position::{CoordinateError, GeolocationAccessError, Position};
pub use provider::check_location_permission;
