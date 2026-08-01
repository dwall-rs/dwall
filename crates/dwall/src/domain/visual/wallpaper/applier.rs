//! Wallpaper application logic for selecting and applying wallpapers

use std::collections::HashMap;

use crate::DwallResult;

#[cfg(windows)]
use crate::DisplayMonitor;

use crate::domain::visual::{MonitorProvider, WallpaperProvider};

/// Applies wallpapers to monitors based on solar position
///
/// This struct holds references to separate wallpaper and monitor providers,
/// following the Single Responsibility Principle.
pub(crate) struct WallpaperApplier<W: WallpaperProvider, M: MonitorProvider> {
    wallpaper_provider: W,
    monitor_provider: M,
}

impl<W: WallpaperProvider, M: MonitorProvider> WallpaperApplier<W, M> {
    /// Creates a new WallpaperApplier with the given providers
    pub(crate) fn new(wallpaper_provider: W, monitor_provider: M) -> Self {
        Self {
            wallpaper_provider,
            monitor_provider,
        }
    }

    /// Returns a reference to the underlying wallpaper provider
    pub(crate) fn setter(&self) -> &W {
        &self.wallpaper_provider
    }

    /// Returns a reference to the underlying monitor provider
    pub(crate) fn monitor(&self) -> &M {
        &self.monitor_provider
    }

    /// Lists all available monitors
    pub(crate) fn list_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        self.monitor_provider.get_monitors()
    }
}
