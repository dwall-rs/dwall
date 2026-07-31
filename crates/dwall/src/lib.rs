pub mod config;
pub mod core;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod lazy;
pub mod utils;

#[macro_use]
extern crate logging;

// Re-export core functionality
pub use config::Config;
pub use core::daemon::DaemonApplication;
pub use error::DwallResult;
pub use lazy::{DWALL_CACHE_DIR, DWALL_CONFIG_DIR, DWALL_LOG_DIR};

// Re-export domain types
pub use domain::geography::Position;
pub use domain::visual::{ThemeValidator, apply_solar_theme};

// Re-export infrastructure types
#[cfg(windows)]
pub use infrastructure::platform::windows::display::{DisplayMonitor, DisplayMonitorProvider};

// Backwards compatibility aliases
pub use domain::geography::CoordinateError;
pub use domain::geography::Position as GeographicPosition;
pub use domain::time::solar_calculator::SolarAngle;
pub use domain::visual::ColorScheme;

#[cfg(windows)]
pub use infrastructure::platform::{RegistryError, RegistryKey};
