//! Monitor enumeration abstraction.

use std::collections::HashMap;
use std::rc::Rc;

use crate::DwallResult;
use crate::platform::DisplayMonitor;

/// One snapshot of the attached monitors, keyed by device path.
///
/// Shared rather than handed out by value: the daemon asks for the list several times
/// per cycle — the cycle itself, the wallpaper setter once per monitor, and every
/// schedule probe — and a clone copied every device path twice (the key and the
/// monitor's own field) on each of those calls.
pub type MonitorSnapshot = Rc<HashMap<String, DisplayMonitor>>;

/// Trait for providing monitor/display information.
///
/// Implemented by the platform display backend (Windows display config APIs).
pub trait MonitorProvider {
    /// The monitors as of the last refresh, shared with the caller.
    fn get_monitors(&self) -> DwallResult<MonitorSnapshot>;

    /// Forces a refresh of monitor information.
    fn refresh_monitors(&self) -> DwallResult<MonitorSnapshot>;

    /// Detects if monitor configuration has changed since last check
    fn has_configuration_changed(&self) -> DwallResult<bool>;
}
