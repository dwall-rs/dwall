//! Monitor enumeration abstraction.

use std::collections::HashMap;

use crate::DwallResult;
use crate::platform::DisplayMonitor;

/// Trait for providing monitor/display information.
///
/// Implemented by the platform display backend (Windows display config APIs).
pub trait MonitorProvider {
    /// Retrieves all currently available monitors
    fn get_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>>;

    /// Forces a refresh of monitor information
    fn refresh_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>>;

    /// Detects if monitor configuration has changed since last check
    fn has_configuration_changed(&self) -> DwallResult<bool>;
}
