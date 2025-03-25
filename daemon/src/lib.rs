mod color_mode;
pub mod config;
pub mod error;
mod lazy;
mod log;
pub mod monitor;
mod position;
mod solar;
mod theme;
mod utils;

#[macro_use]
extern crate tracing;

pub use color_mode::ColorMode;
pub use error::{DwallError, DwallResult};
pub use lazy::{DWALL_CACHE_DIR, DWALL_CONFIG_DIR};
pub use log::setup_logging;
pub use theme::{apply_theme, ThemeValidator};
