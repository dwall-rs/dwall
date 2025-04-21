mod device_interface;
mod display_config;
pub mod error;

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use windows::{
    core::HSTRING,
    Win32::{
        Devices::Display::GUID_DEVINTERFACE_MONITOR,
        System::Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_ALL},
        UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
    },
};

use crate::{error::DwallResult, utils::string::WideStringRead};

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

/// Manager for wallpaper operations
pub struct WallpaperManager {
    /// Windows Desktop Wallpaper COM interface
    desktop_wallpaper: IDesktopWallpaper,
    /// Monitor information provider
    monitor_provider: MonitorInfoProvider,
    /// Flag indicating whether CoUninitialize should be called on drop
    com_initialized: bool,
}

impl WallpaperManager {
    /// Creates a new WallpaperManager instance
    fn new() -> DwallResult<Self> {
        // Continue execution even if initialization fails, as it might have been initialized elsewhere
        let com_initialized = unsafe {
            // Attempt to initialize COM
            let result = CoInitialize(None);
            // If returns S_OK (0), means initialization succeeded and needs cleanup
            // If returns S_FALSE, means already initialized and no cleanup needed
            result.0 == 0
        };

        let desktop_wallpaper: IDesktopWallpaper = unsafe {
            CoCreateInstance(&DesktopWallpaper as *const _, None, CLSCTX_ALL).map_err(|e| {
                error!(
                    error = ?e,
                    "Failed to create desktop wallpaper COM instance"
                );
                e
            })?
        };

        Ok(Self {
            desktop_wallpaper,
            monitor_provider: MonitorInfoProvider::new(),
            com_initialized,
        })
    }

    /// Checks if the wallpaper is already set for a specific monitor
    fn is_wallpaper_already_set(
        &self,
        monitor_path: &HSTRING,
        wallpaper_path: &HSTRING,
    ) -> DwallResult<bool> {
        trace!(
            monitor_path = %monitor_path,
            "Checking existing wallpaper for monitor"
        );

        let current_wallpaper_ptr =
            match unsafe { self.desktop_wallpaper.GetWallpaper(monitor_path) } {
                Ok(path) => path,
                Err(e) => {
                    error!(
                        error = %e,
                        monitor_path = %monitor_path,
                        "Failed to retrieve current wallpaper for monitor"
                    );
                    return Err(e.into());
                }
            };

        let current_wallpaper_path = unsafe { current_wallpaper_ptr.to_hstring() };

        let is_same = *wallpaper_path == current_wallpaper_path;

        trace!(
            current_wallpaper_path = %current_wallpaper_path,
            new_wallpaper_path = %wallpaper_path,
            is_same = is_same,
            "Wallpaper comparison result"
        );

        Ok(is_same)
    }

    /// Sets wallpaper for a specific monitor
    async fn set_wallpaper(
        &self,
        monitor_id: &str,
        wallpaper_path: &std::path::Path,
    ) -> DwallResult<()> {
        // Verify wallpaper path exists
        if !wallpaper_path.exists() {
            error!(
                image_path = %wallpaper_path.display(),
                "Image path does not exist. Cannot proceed with wallpaper setting."
            );
            return Err(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Image file not found").into(),
            );
        }

        // Get all monitors
        let monitors = self.monitor_provider.get_monitors().await?;

        // Find monitor with specified ID
        let monitor = monitors.get(monitor_id).ok_or_else(|| {
            error!(
                monitor_id = monitor_id,
                "Monitor with specified ID not found"
            );
            std::io::Error::new(std::io::ErrorKind::NotFound, "Monitor not found")
        })?;

        // Convert wallpaper path to HSTRING
        let wallpaper_path = HSTRING::from(wallpaper_path);
        let device_path = HSTRING::from(&monitor.device_path);

        if self.is_wallpaper_already_set(&device_path, &wallpaper_path)? {
            info!(
                monitor_id = monitor_id,
                wallpaper_path = %wallpaper_path,
                "Wallpaper already set for monitor, skipping"
            );
            return Ok(());
        };

        // Set wallpaper
        let result = unsafe {
            self.desktop_wallpaper
                .SetWallpaper(&device_path, &wallpaper_path)
                .map_err(|e| {
                    error!(
                        error = ?e,
                        monitor_id = monitor_id,
                        wallpaper_path = %wallpaper_path,
                        "Failed to set wallpaper for monitor"
                    );
                    e
                })
        };

        // If setting wallpaper failed, try to refresh monitor info and retry once
        if result.is_err() {
            warn!("Refreshing monitor information and retrying...");
            self.monitor_provider.refresh_monitors().await?;

            let monitors = self.monitor_provider.get_monitors().await?;
            if let Some(monitor) = monitors.get(monitor_id) {
                let device_path = HSTRING::from(&monitor.device_path);
                unsafe {
                    self.desktop_wallpaper
                        .SetWallpaper(&device_path, &wallpaper_path)
                        .map_err(|e| {
                            error!(
                                error = ?e,
                                monitor_id = monitor_id,
                                wallpaper_path = %wallpaper_path,
                                "Failed to set wallpaper for monitor after refresh"
                            );
                            e
                        })?
                };
            }
        } else {
            // Original call succeeded
            result?;
        }

        info!(
            monitor_id = monitor_id,
            wallpaper_path = %wallpaper_path,
            "Successfully set wallpaper for monitor"
        );

        Ok(())
    }

    /// Gets all available monitors with caching
    async fn get_monitors(&self) -> DwallResult<HashMap<String, Monitor>> {
        self.monitor_provider.get_monitors().await
    }

    /// Forces a refresh of monitor information
    async fn refresh_monitors(&self) -> DwallResult<HashMap<String, Monitor>> {
        self.monitor_provider.refresh_monitors().await
    }

    /// Detects if monitor configuration has changed
    async fn has_monitor_config_changed(&self) -> DwallResult<bool> {
        self.monitor_provider.has_monitor_config_changed().await
    }
}

impl Drop for WallpaperManager {
    fn drop(&mut self) {
        // Only uninitialize COM if we successfully initialized it
        if self.com_initialized {
            unsafe {
                CoUninitialize();
            }
            debug!("COM resources released");
        }
    }
}

/// For backward compatibility with existing code
/// This will be deprecated in future versions
pub struct MonitorManager {
    /// Wallpaper manager instance
    wallpaper_manager: WallpaperManager,
}

impl MonitorManager {
    /// Creates a new MonitorManager instance
    pub fn new() -> DwallResult<Self> {
        Ok(Self {
            wallpaper_manager: WallpaperManager::new()?,
        })
    }

    /// Sets wallpaper for a specific monitor
    pub(crate) async fn set_wallpaper(
        &self,
        monitor_id: &str,
        wallpaper_path: &std::path::Path,
    ) -> DwallResult<()> {
        self.wallpaper_manager
            .set_wallpaper(monitor_id, wallpaper_path)
            .await
    }

    /// Gets all available monitors with caching
    pub async fn get_monitors(&self) -> DwallResult<HashMap<String, Monitor>> {
        self.wallpaper_manager.get_monitors().await
    }

    /// Forces a refresh of monitor information
    pub async fn refresh_monitors(&self) -> DwallResult<HashMap<String, Monitor>> {
        self.wallpaper_manager.refresh_monitors().await
    }

    /// Detects if monitor configuration has changed
    pub async fn has_monitor_config_changed(&self) -> DwallResult<bool> {
        self.wallpaper_manager.has_monitor_config_changed().await
    }
}

/// Queries monitor information from the system
fn query_monitor_info() -> DwallResult<HashMap<String, Monitor>> {
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
