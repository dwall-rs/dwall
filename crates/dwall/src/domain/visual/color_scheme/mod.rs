//! Color scheme detection and application for automatic light/dark mode switching

mod applier;
mod constants;
mod daylight_state;
mod engine;
mod threshold;

use std::fmt;

use serde::Deserialize;

// Re-exports for backward compatibility
pub(crate) use applier::ColorSchemeApplier;
pub use daylight_state::DaylightState;
pub use threshold::ThresholdConfig;

/// Trait for managing system color scheme
///
/// Implementations can interact with various platform APIs:
/// - Windows Registry (AppsUseLightTheme/SystemUsesLightTheme)
/// - macOS NSAppearance
/// - Linux desktop environment settings
pub trait ColorSchemeProvider {
    /// Retrieves the current system color scheme
    ///
    /// Returns the current color scheme (Light or Dark) as reported by the system.
    fn get_current_scheme(&self) -> crate::error::DwallResult<ColorScheme>;

    /// Sets the system color scheme
    ///
    /// # Arguments
    /// * scheme - The color scheme to apply (Light or Dark)
    ///
    /// This should also notify the system about the change so that
    /// applications can update their appearance.
    fn set_color_scheme(&self, scheme: ColorScheme) -> crate::error::DwallResult<()>;
}

#[derive(Debug, PartialEq, Deserialize, Copy, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum ColorScheme {
    Light,
    Dark,
}

impl ColorScheme {
    #[inline]
    const fn as_u32(&self) -> u32 {
        match self {
            ColorScheme::Light => 1,
            ColorScheme::Dark => 0,
        }
    }

    #[inline]
    pub(crate) const fn to_le_bytes(self) -> [u8; 4] {
        self.as_u32().to_le_bytes()
    }
}

impl fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorScheme::Light => write!(f, "Light"),
            ColorScheme::Dark => write!(f, "Dark"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_scheme_display_light() {
        assert_eq!(ColorScheme::Light.to_string(), "Light");
    }

    #[test]
    fn color_scheme_display_dark() {
        assert_eq!(ColorScheme::Dark.to_string(), "Dark");
    }

    #[test]
    fn color_scheme_to_le_bytes_light() {
        assert_eq!(ColorScheme::Light.to_le_bytes(), [1, 0, 0, 0]);
    }

    #[test]
    fn color_scheme_to_le_bytes_dark() {
        assert_eq!(ColorScheme::Dark.to_le_bytes(), [0, 0, 0, 0]);
    }

    #[test]
    fn color_scheme_equality() {
        assert_eq!(ColorScheme::Light, ColorScheme::Light);
        assert_eq!(ColorScheme::Dark, ColorScheme::Dark);
        assert_ne!(ColorScheme::Light, ColorScheme::Dark);
    }
}
