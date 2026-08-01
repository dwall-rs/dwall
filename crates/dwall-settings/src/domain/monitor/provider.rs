//! Monitor provider (domain knowledge: display management)

use std::collections::HashMap;

use dwall::domain::visual::MonitorProvider as _;
use dwall::{DisplayMonitor, DisplayMonitorProvider, DwallResult};

/// Provides information about connected monitors
pub struct MonitorProvider;

impl MonitorProvider {
    /// Get all available monitors
    pub fn get_monitors() -> DwallResult<HashMap<String, DisplayMonitor>> {
        let provider = DisplayMonitorProvider::new();
        provider.refresh_monitors()
    }
}
