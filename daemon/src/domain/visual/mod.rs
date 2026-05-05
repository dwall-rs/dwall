pub mod color_scheme;
pub mod theme_processor;
pub(crate) mod wallpaper;

// Re-export commonly used types
pub use color_scheme::ColorScheme;
pub use theme_processor::{SolarThemeValidator, ThemeProcessingError, apply_solar_theme};
