//! Windows platform integration.

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use windows::{
    autostart::{
        install_autostart, is_autostart_installed, registered_autostart_command,
        uninstall_autostart,
    },
    color_scheme::ColorSchemeScheduler,
    geolocation::Positioner,
    monitor::{DisplayError, DisplayMonitor, DisplayMonitorProvider},
    registry::{RegistryError, RegistryKey},
    single_instance::{DaemonInstance, acquire_daemon_instance, daemon_is_running},
    wallpaper::WallpaperError,
};

#[cfg(windows)]
pub(crate) use windows::wallpaper::WallpaperSetter;
