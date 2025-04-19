//! Position module for handling geolocation functionality
//!
//! This module provides functionality for retrieving and managing geographical positions,
//! including coordinate validation, geolocation access, and position caching.

mod geolocation_access;
mod position_manager;
mod position_types;
mod windows_api_helper;

// Re-export public items
pub use geolocation_access::{check_location_permission, GeolocationAccessError};
pub(crate) use position_manager::PositionManager;
pub use position_types::{Position, PositionError};
