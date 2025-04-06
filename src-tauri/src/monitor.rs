use std::collections::HashMap;

use dwall::{
    monitor::{Monitor, MonitorInfoProvider},
    DwallResult,
};

pub async fn get_monitors() -> DwallResult<HashMap<String, Monitor>> {
    MonitorInfoProvider::new().get_monitors().await
}
