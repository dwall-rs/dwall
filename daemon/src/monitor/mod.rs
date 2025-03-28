mod device_interface;
mod display_config;
pub mod error;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use windows::Win32::{
    Devices::Display::GUID_DEVINTERFACE_MONITOR,
    System::Com::{CoCreateInstance, CLSCTX_ALL},
    UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
};
use windows_strings::HSTRING;

use crate::{error::DwallResult, utils::string::WideStringRead};

#[derive(Debug, Serialize, Deserialize)]
pub struct Monitor {
    pub id: String,
    pub device_path: String,
    pub display_name: String,
    pub index: u32,
}

pub struct MonitorManager {
    desktop_wallpaper: IDesktopWallpaper,
}

impl MonitorManager {
    pub fn new() -> DwallResult<Self> {
        let desktop_wallpaper: IDesktopWallpaper = unsafe {
            CoCreateInstance(&DesktopWallpaper as *const _, None, CLSCTX_ALL).map_err(|e| {
                error!(
                    error = ?e,
                    "Failed to create desktop wallpaper COM instance"
                );
                e
            })?
        };

        Ok(Self { desktop_wallpaper })
    }

    pub(crate) fn set_wallpaper(
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
        let monitors = query_monitor_info()?;

        // Find monitor with specified ID
        let monitor = monitors.get(monitor_id).ok_or_else(|| {
            error!(
                monitor_id = monitor_id,
                "Monitor with specified ID not found"
            );
            std::io::Error::new(std::io::ErrorKind::NotFound, "Monitor not found")
        })?;

        // Convert wallpaper path to HSTRING
        let wallpaper_path = windows::core::HSTRING::from(wallpaper_path);

        let device_path = HSTRING::from(&monitor.device_path);

        // Set wallpaper
        unsafe {
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
                })?;
        }

        info!(
            monitor_id = monitor_id,
            wallpaper_path = %wallpaper_path,
            "Successfully set wallpaper for monitor"
        );

        Ok(())
    }

    pub fn get_monitors(&self) -> DwallResult<HashMap<String, MonitorInfo>> {
        query_monitor_info()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub friendly_name: String,
    pub device_path: String,
}

fn query_monitor_info() -> DwallResult<HashMap<String, MonitorInfo>> {
    info!("Querying monitor info");
    let mut monitor_info = HashMap::new();

    for display_path in display_config::query_display_paths()?.into_iter() {
        let target_info =
            display_config::query_target_name(display_path.adapter_id, display_path.target_id)?;
        let device_path = target_info.monitorDevicePath.to_string();

        match device_interface::query_device_friendly_name(&device_path, &GUID_DEVINTERFACE_MONITOR)
        {
            Ok(friendly_name) => {
                info!(
                    friendly_name = friendly_name,
                    "Succesfull to get friendly name"
                );
                monitor_info.insert(
                    device_path.clone(),
                    MonitorInfo {
                        friendly_name,
                        device_path,
                    },
                );
            }
            Err(e) => {
                warn!(error = ?e, "Failed to get friendly name for device path");
            }
        }
    }

    info!("Found {} monitors in total", monitor_info.len());
    Ok(monitor_info)
}
