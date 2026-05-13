pub mod color_scheme;
pub mod theme_processor;
pub(crate) mod wallpaper;

// Re-export commonly used types
pub use color_scheme::ColorScheme;
pub use theme_processor::{ThemeError, ThemeValidator, apply_solar_theme};
