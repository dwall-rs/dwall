pub mod color_scheme;
pub mod theme_processor;
pub mod wallpaper;

// Re-export commonly used types
pub use color_scheme::{determine_color_mode, set_color_mode, ColorMode, ColorSchemeManager};
pub use theme_processor::{apply_theme, ThemeError, ThemeValidator};
pub use wallpaper::WallpaperSelector;
