use std::collections::HashMap;

use dwall::{
    monitor::{DisplayMonitor, DisplayMonitorProvider},
    DwallResult,
};

pub async fn get_monitors() -> DwallResult<HashMap<String, DisplayMonitor>> {
    DisplayMonitorProvider::new().get_monitors().await
}
