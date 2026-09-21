pub mod color_scheme;
pub mod config;
pub mod daemon;
pub mod error;
pub mod lazy;
pub mod monitor;
pub mod platform;
pub mod solar;
pub mod theme;
pub mod utils;
pub mod wallpaper;

mod theme_engine;

#[macro_use]
extern crate logging;

// Re-export core functionality
pub use config::Config;
pub use daemon::DaemonApplication;
pub use error::DwallResult;
pub use lazy::{DWALL_CACHE_DIR, DWALL_CONFIG_DIR, DWALL_LOG_DIR};

// Re-export domain types
pub use color_scheme::{ColorScheme, ColorSchemeProvider, DaylightState, ThresholdConfig};
pub use monitor::MonitorProvider;
pub use solar::{
    CoordinateError, GeographicPositionProvider, Position, PositionProvider, SolarAngle,
    SolarPosition,
};
pub use theme::{ThemeError, ThemeValidator};
pub use wallpaper::{WallpaperProvider, WallpaperSelector};

// Re-export platform types
#[cfg(windows)]
pub use platform::{
    DisplayMonitor, DisplayMonitorProvider, Positioner, RegistryError, RegistryKey,
};

// Backwards compatibility aliases
pub use solar::Position as GeographicPosition;
