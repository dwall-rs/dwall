#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use windows::{
    color_scheme::ColorSchemeScheduler,
    geolocation::Positioner,
    registry_client::{RegistryError, RegistryKey},
};
