use std::collections::HashMap;

use crate::{DisplayMonitor, DwallResult};

/// Trait for providing monitor/display information
///
/// Implementations can query monitor information from various sources such as:
/// - Windows EnumDisplayDevices/EnumDisplayMonitors APIs
/// - X11 RandR on Linux
/// - Wayland wlr-output-management on Linux
pub trait MonitorProvider {
    /// Retrieves all currently available monitors
    ///
    /// Returns a map of monitor ID to monitor information.
    fn get_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>>;

    /// Forces a refresh of monitor information
    ///
    /// Clears any cached data and re-queries the system.
    fn refresh_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>>;

    /// Detects if monitor configuration has changed since last check
    ///
    /// Returns `true` if monitors were added, removed, or changed.
    fn has_configuration_changed(&self) -> DwallResult<bool>;
}
