use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use windows::Win32::Devices::Display::GUID_DEVINTERFACE_MONITOR;

use crate::{error::DwallResult, utils::string::WideStringRead};

use super::{device_interface, display_config};

/// Cache expiration time in seconds - increased to reduce system API calls
const CACHE_EXPIRY_SECONDS: u64 = 300; // 5 minutes instead of 30 seconds

/// Unified monitor information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayMonitor {
    // /// Unique identifier for the monitor (device path)
    // pub id: String,
    /// Unique device path identifier
    device_path: String,
    /// User-friendly display name
    friendly_name: String,
    /// Monitor position index in display configuration
    position_index: Option<u32>,
}

impl DisplayMonitor {
    fn new(device_path: String, friendly_name: String, position_index: Option<u32>) -> Self {
        Self {
            device_path,
            friendly_name,
            position_index,
        }
    }

    pub(super) fn device_path(&self) -> &str {
        &self.device_path
    }

    pub fn friendly_name(&self) -> &str {
        &self.friendly_name
    }

    pub fn position_index(&self) -> Option<u32> {
        self.position_index
    }
}

/// Cache structure for monitor information
struct MonitorInfoCache {
    /// Cached monitor data (key: device_path)
    data: HashMap<String, DisplayMonitor>,
    /// Last cache update timestamp
    updated_at: Instant,
    /// Cached expiration duration
    expiry_duration: Duration,
}

impl MonitorInfoCache {
    /// Creates a new empty cache
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            updated_at: Instant::now(),
            expiry_duration: Duration::from_secs(CACHE_EXPIRY_SECONDS),
        }
    }

    /// Checks if the cache is expired or empty
    fn is_valid(&self) -> bool {
        !self.data.is_empty() && self.updated_at.elapsed() <= self.expiry_duration
    }

    /// Updates the cache with new monitor information
    fn update(&mut self, monitors: HashMap<String, DisplayMonitor>) {
        self.data = monitors;
        self.updated_at = Instant::now();
    }
}

/// Provider for display monitor information with caching
pub struct DisplayMonitorProvider {
    /// Cached monitor information
    cache: RwLock<MonitorInfoCache>,
}

impl Default for DisplayMonitorProvider {
    fn default() -> Self {
        Self {
            cache: RwLock::new(MonitorInfoCache::new()),
        }
    }
}

impl DisplayMonitorProvider {
    /// Creates a new MonitorInfoProvider instance
    pub fn new() -> Self {
        Default::default()
    }

    /// Gets all available monitors with caching
    pub async fn get_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        // Try to read from cache first
        {
            let cache = self.cache.read().await;
            if cache.is_valid() {
                debug!("Using cached monitor information");
                return Ok(cache.data.clone());
            }
        }

        // Cache is expired or empty, refresh it
        self.refresh_monitors().await
    }

    /// Forces a refresh of monitor information
    pub async fn refresh_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        debug!("Refreshing monitor information");
        let monitors = fetch_system_monitors()?;

        {
            let mut cache = self.cache.write().await;
            cache.update(monitors.clone());
        }

        Ok(monitors)
    }

    /// Detects if monitor configuration has changed
    pub async fn has_configuration_changed(&self) -> DwallResult<bool> {
        let current_monitors = fetch_system_monitors()?;

        // Compare with cached monitors
        let cache = self.cache.read().await;

        // Quick check: different monitor count means configuration changed
        if current_monitors.len() != cache.data.len() {
            return Ok(true);
        }

        // Check if any monitor device paths are different
        for device_path in current_monitors.keys() {
            if !cache.data.contains_key(device_path) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Queries monitor information from the system
pub(crate) fn fetch_system_monitors() -> DwallResult<HashMap<String, DisplayMonitor>> {
    debug!("Fetching monitor information from system");
    let mut monitors = HashMap::new();

    for (index, display_path) in display_config::query_display_paths()?
        .into_iter()
        .enumerate()
    {
        let target_info =
            display_config::query_target_name(display_path.adapter_id, display_path.target_id)?;
        let device_path = target_info.monitorDevicePath.to_string();

        // Try to get friendly name, use fallback if unavailable
        let friendly_name = match device_interface::query_device_friendly_name(
            &device_path,
            &GUID_DEVINTERFACE_MONITOR,
        ) {
            Ok(name) => name,
            Err(e) => {
                warn!(error = %e, "Failed to get friendly name, using fallback");
                format!("Display {}", index + 1) // Fallback name
            }
        };

        monitors.insert(
            device_path.clone(),
            DisplayMonitor::new(device_path, friendly_name, Some(index as u32)),
        );
    }

    let monitor_count = monitors.len();
    info!("Found {} active monitor(s)", monitor_count);

    Ok(monitors)
}
