use std::collections::HashMap;

use dwall::{
    monitor::{MonitorInfo, MonitorManager},
    DwallResult,
};

pub fn get_monitors() -> DwallResult<HashMap<String, MonitorInfo>> {
    let monitors = MonitorManager::new()?.get_monitors()?;
    Ok(monitors)
}
