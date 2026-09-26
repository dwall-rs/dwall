//! Windows platform integration.

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::{
    color_scheme::ColorSchemeScheduler,
    geolocation::Positioner,
    monitor::{DisplayError, DisplayMonitor, DisplayMonitorProvider},
    registry::{RegistryError, RegistryKey},
    wallpaper::WallpaperError,
};

#[cfg(windows)]
pub(crate) use windows::wallpaper::WallpaperSetter;
