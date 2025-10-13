pub mod config;
pub mod core;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod lazy;
pub mod utils;

#[macro_use]
extern crate tracing;

// Re-export core functionality
pub use config::Config;
pub use core::daemon::DaemonApplication;
pub use error::DwallResult;
pub use lazy::{DWALL_CACHE_DIR, DWALL_CONFIG_DIR};
pub use utils::logging::setup_logging;

// Re-export domain types
pub use domain::geography::Coordinate;
pub use domain::visual::{apply_solar_theme, SolarThemeValidator};

// Re-export infrastructure types
pub use infrastructure::display::{DisplayMonitor, DisplayMonitorProvider};
pub use infrastructure::filesystem::{read_config_file, write_config_file, ConfigManager};
pub use infrastructure::platform::windows::{RegistryError, RegistryKey};

// Backwards compatibility aliases
pub use domain::geography::Coordinate as GeographicPosition;
pub use domain::geography::CoordinateError;
pub use domain::geography::GeolocationAccessError;
pub use domain::visual::ColorMode;
