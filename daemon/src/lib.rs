mod color_mode;
pub mod config;
pub mod error;
mod geo;
mod lazy;
mod log;
mod solar;
mod theme;

#[macro_use]
extern crate tracing;

pub use error::{DwallError, DwallResult};
pub use lazy::APP_CONFIG_DIR;
pub use log::setup_logging;
pub use theme::{apply_theme, ThemeValidator, THEMES_DIR};
