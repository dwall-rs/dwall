pub mod color_scheme;
pub mod monitor;
pub mod theme;
pub mod wallpaper;

// Re-export commonly used types
pub use color_scheme::{ColorScheme, ColorSchemeProvider, DaylightState, ThresholdConfig};
pub use monitor::MonitorProvider;
pub use theme::engine::apply_solar_theme;
pub use theme::{ThemeError, ThemeValidator};
pub use wallpaper::WallpaperProvider;
