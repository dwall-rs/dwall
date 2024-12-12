mod color_mode;
pub mod config;
pub mod error;
mod lazy;
mod log;
mod position;
mod solar;
mod theme;

#[macro_use]
extern crate tracing;

pub use color_mode::ColorMode;
pub use error::{DwallError, DwallResult};
pub use lazy::APP_CONFIG_DIR;
pub use log::setup_logging;
pub use theme::{apply_theme, ThemeValidator};
