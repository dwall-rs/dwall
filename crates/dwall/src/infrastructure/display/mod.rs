pub(crate) mod device_query;
pub(crate) mod display_query;
pub mod error;
pub mod monitor_manager;
pub mod wallpaper_setter;

// Re-export commonly used types
pub use error::DisplayError;
pub use monitor_manager::{DisplayMonitor, DisplayMonitorProvider};
pub(crate) use wallpaper_setter::WallpaperError;
