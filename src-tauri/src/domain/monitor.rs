//! Monitor domain module
//!
//! This module contains the core business logic related to monitor management.

use std::collections::HashMap;

use dwall::{DisplayMonitor, DisplayMonitorProvider, DwallResult};

/// Get all available monitors
pub async fn get_monitors() -> DwallResult<HashMap<String, DisplayMonitor>> {
    DisplayMonitorProvider::new().get_monitors().await
}
