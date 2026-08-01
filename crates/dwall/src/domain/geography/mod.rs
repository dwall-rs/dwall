//! Position provider trait abstraction
//!
//! This trait abstracts the source of geographic position data,
//! enabling easier testing and potential future platform support.

mod error;
pub mod position;
pub mod provider;

// Re-export commonly used types
pub use error::GeolocationAccessError;
pub use position::{CoordinateError, Position};
// pub use provider::check_location_permission;

use crate::error::DwallResult;

/// Trait for providing geographic position data
///
/// Implementations can fetch position from various sources such as:
/// - Windows Geolocator API
/// - Manual configuration
/// - IP-based geolocation
/// - Cached position data
pub trait PositionProvider {
    /// Retrieves the current geographic position
    ///
    /// Returns the current position based on the provider's configuration,
    /// or an error if the position cannot be determined.
    fn get_current_position(&self) -> DwallResult<Position>;

    /// Checks if the application has permission to access location
    ///
    /// Returns Ok(()) if permission is granted, or an error if denied or unspecified
    fn check_location_permission(&self) -> DwallResult<()>;
}
