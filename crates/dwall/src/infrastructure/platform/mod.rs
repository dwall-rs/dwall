#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::{
    color_scheme::ColorSchemeScheduler,
    display::{DisplayMonitor, DisplayMonitorProvider},
    geolocation::Positioner,
    registry_client::{RegistryError, RegistryKey},
};
