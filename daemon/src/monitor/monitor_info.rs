use std::sync::Arc;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use windows::Win32::Devices::Display::GUID_DEVINTERFACE_MONITOR;

use crate::{error::DwallResult, utils::string::WideStringRead};

use super::{device_interface, display_config};

/// Cache expiration time in seconds
const MONITOR_CACHE_EXPIRY: u64 = 30;

/// Unified monitor information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    /// Unique identifier for the monitor (device path)
    pub id: String,
    /// Full device path
    pub device_path: String,
    /// User-friendly display name
    pub friendly_name: String,
    /// Monitor index (position in the display configuration)
    pub index: Option<u32>,
}

/// Cache structure for monitor information
struct MonitorCache {
    /// Cached monitor information
    monitors: HashMap<String, Monitor>,
    /// Last update timestamp
    last_update: Instant,
}

impl MonitorCache {
    /// Creates a new empty cache
    fn new() -> Self {
        Self {
            monitors: HashMap::new(),
            last_update: Instant::now(),
        }
    }

    /// Checks if the cache is expired
    fn is_expired(&self) -> bool {
        self.last_update.elapsed() > Duration::from_secs(MONITOR_CACHE_EXPIRY)
    }

    /// Updates the cache with new monitor information
    fn update(&mut self, monitors: HashMap<String, Monitor>) {
        self.monitors = monitors;
        self.last_update = Instant::now();
    }
}

/// Provider for monitor information with caching
pub struct MonitorInfoProvider {
    /// Cache for monitor information
    cache: Arc<RwLock<MonitorCache>>,
}

impl Default for MonitorInfoProvider {
    fn default() -> Self {
        Self {
            cache: Arc::new(RwLock::new(MonitorCache::new())),
        }
    }
}

impl MonitorInfoProvider {
    /// Creates a new MonitorInfoProvider instance
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets all available monitors with caching
    pub async fn get_monitors(&self) -> DwallResult<HashMap<String, Monitor>> {
        // Try to read from cache first
        {
            let cache = self.cache.read().await;
            if !cache.is_expired() && !cache.monitors.is_empty() {
                debug!("Using cached monitor information");
                return Ok(cache.monitors.clone());
            }
        }

        // Cache is expired or empty, refresh it
        self.refresh_monitors().await
    }

    /// Forces a refresh of monitor information
    pub async fn refresh_monitors(&self) -> DwallResult<HashMap<String, Monitor>> {
        debug!("Refreshing monitor information");
        let monitors = query_monitor_info()?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.update(monitors.clone());
        }

        Ok(monitors)
    }

    /// Detects if monitor configuration has changed
    pub async fn has_monitor_config_changed(&self) -> DwallResult<bool> {
        let current_monitors = query_monitor_info()?;

        // Compare with cached monitors
        let cache = self.cache.read().await;
        let cached_monitors = &cache.monitors;

        // Check if number of monitors changed
        if current_monitors.len() != cached_monitors.len() {
            return Ok(true);
        }

        // Check if any monitor IDs changed
        for id in current_monitors.keys() {
            if !cached_monitors.contains_key(id) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Queries monitor information from the system
pub(crate) fn query_monitor_info() -> DwallResult<HashMap<String, Monitor>> {
    debug!("Querying monitor information from system");
    let mut monitors = HashMap::new();

    for (index, display_path) in display_config::query_display_paths()?
        .into_iter()
        .enumerate()
    {
        let target_info =
            display_config::query_target_name(display_path.adapter_id, display_path.target_id)?;
        let device_path = target_info.monitorDevicePath.to_string();

        // Try to get friendly name, use device path as fallback
        let friendly_name = match device_interface::query_device_friendly_name(
            &device_path,
            &GUID_DEVINTERFACE_MONITOR,
        ) {
            Ok(name) => {
                debug!(friendly_name = name, "Successfully retrieved friendly name");
                name
            }
            Err(e) => {
                warn!(error = ?e, "Failed to get friendly name, using fallback");
                // Use a fallback name based on the device path
                format!("Display {}", index + 1)
            }
        };

        monitors.insert(
            device_path.clone(),
            Monitor {
                id: device_path.clone(),
                device_path,
                friendly_name,
                index: Some(index as u32),
            },
        );
    }

    info!(monitors = ?monitors, "Found all active monitors");
    Ok(monitors)
}
