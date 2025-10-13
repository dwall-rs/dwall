use std::collections::HashMap;

use dwall::{DisplayMonitor, DisplayMonitorProvider, DwallResult};

pub async fn get_monitors() -> DwallResult<HashMap<String, DisplayMonitor>> {
    DisplayMonitorProvider::new().get_monitors().await
}
