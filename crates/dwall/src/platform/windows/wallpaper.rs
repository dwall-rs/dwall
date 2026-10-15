//! Windows implementation of [`WallpaperProvider`].

use std::{collections::HashMap, path::Path, time::Duration, time::Instant};

use windows::{
    Foundation::Uri,
    Storage::{IStorageFile, StorageFile},
    System::UserProfile::LockScreen,
    Win32::{
        System::Com::{CLSCTX_ALL, CoCreateInstance, CoInitialize, CoTaskMemFree, CoUninitialize},
        UI::Shell::{DesktopWallpaper, IDesktopWallpaper},
    },
    core::{HSTRING, Interface, PWSTR},
};
use windows_future::{AsyncStatus, IAsyncAction, IAsyncOperation};

use crate::monitor::MonitorProvider;
use crate::wallpaper::WallpaperProvider;
use crate::{DwallResult, platform::DisplayMonitor};

use super::monitor::DisplayMonitorProvider;

/// Wallpaper operation errors
#[derive(Debug, thiserror::Error)]
pub enum WallpaperError {
    #[error("Failed to create desktop wallpaper COM instance: {0}")]
    Instance(windows::core::Error),
    #[error("Failed to retrieve current wallpaper for monitor: {0}")]
    GetWallpaper(windows::core::Error),
    #[error("Failed to set wallpaper for monitor: {0}")]
    SetWallpaper(windows::core::Error),
    #[error("Monitor with specified ID not found: {0}")]
    MonitorNotFound(String),
    #[error("WinRT operation failed: {0}")]
    AsyncFailed(windows::core::Error),
    #[error("Timed out waiting for a WinRT operation after {0:?}")]
    Timeout(Duration),
}

type WallpaperResult<T> = Result<T, WallpaperError>;

/// How long a lock-screen WinRT operation may block the daemon.
const LOCK_SCREEN_TIMEOUT: Duration = Duration::from_secs(30);

/// Poll interval while waiting for a WinRT operation.
const ASYNC_POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Waits for a WinRT `IAsyncOperation` with a bounded timeout.
///
/// `windows-future`'s `get()` waits with `WaitForSingleObject(…, INFINITE)`, so a
/// stuck broker — a lock-screen file on an unreachable network share, a wedged
/// shell — would block the single-threaded daemon forever. Polling `Status()`
/// keeps the wait bounded without installing a completion handler (which would
/// require the result type to be `Send`).
fn wait_bounded<T: windows::core::RuntimeType>(
    operation: &IAsyncOperation<T>,
    timeout: Duration,
) -> WallpaperResult<T> {
    let started = Instant::now();

    loop {
        match operation.Status() {
            Ok(AsyncStatus::Started) => {
                if started.elapsed() >= timeout {
                    return Err(WallpaperError::Timeout(timeout));
                }
                std::thread::sleep(ASYNC_POLL_INTERVAL);
            }
            _ => return operation.GetResults().map_err(WallpaperError::AsyncFailed),
        }
    }
}

/// Bounded wait for a WinRT `IAsyncAction` (no result value).
fn wait_action_bounded(operation: &IAsyncAction, timeout: Duration) -> WallpaperResult<()> {
    let started = Instant::now();

    loop {
        match operation.Status() {
            Ok(AsyncStatus::Started) => {
                if started.elapsed() >= timeout {
                    return Err(WallpaperError::Timeout(timeout));
                }
                std::thread::sleep(ASYNC_POLL_INTERVAL);
            }
            _ => return operation.GetResults().map_err(WallpaperError::AsyncFailed),
        }
    }
}

/// Owns a COM-allocated string handed out by the shell.
///
/// `IDesktopWallpaper::GetWallpaper` fills an `[out] LPWSTR` that the callee
/// allocated with `CoTaskMemAlloc`; the caller is responsible for releasing it.
/// `windows` returns a bare [`PWSTR`] (no `Drop`, no `Free`), so without this
/// guard every query leaks one block — and the query runs once per monitor per
/// cycle.
struct OwnedCoTaskMemString(PWSTR);

impl Drop for OwnedCoTaskMemString {
    fn drop(&mut self) {
        unsafe {
            // SAFETY: the pointer came from `GetWallpaper`, which allocates with
            // `CoTaskMemAlloc`, and it is freed exactly once here.
            CoTaskMemFree(Some(self.0.0 as *const core::ffi::c_void));
        }
    }
}

/// Windows wallpaper setter with COM management
pub(crate) struct WallpaperSetter {
    desktop_wallpaper: IDesktopWallpaper,
    monitor_provider: DisplayMonitorProvider,
    should_cleanup_com: bool,
}

impl WallpaperSetter {
    /// Creates a new WallpaperSetter instance
    pub(crate) fn new() -> WallpaperResult<Self> {
        let should_cleanup_com = unsafe {
            let result = CoInitialize(None);
            result.0 == 0
        };

        let desktop_wallpaper: IDesktopWallpaper = unsafe {
            CoCreateInstance(&DesktopWallpaper as *const _, None, CLSCTX_ALL).map_err(|e| {
                error!(
                    error = %e,
                    "Failed to create desktop wallpaper COM instance"
                );
                WallpaperError::Instance(e)
            })?
        };

        Ok(Self {
            desktop_wallpaper,
            monitor_provider: DisplayMonitorProvider::new(),
            should_cleanup_com,
        })
    }

    /// Sets wallpaper for a specific monitor
    fn set_monitor_wallpaper_internal(
        &self,
        monitor_id: &str,
        image_path: &Path,
    ) -> DwallResult<()> {
        if !image_path.exists() {
            error!(
                image_path = %image_path.display(),
                "Image path does not exist. Cannot proceed with wallpaper setting."
            );
            return Err(
                std::io::Error::new(std::io::ErrorKind::NotFound, "Image file not found").into(),
            );
        }

        let monitors = self.list_available_monitors()?;
        let monitor = monitors.get(monitor_id).ok_or_else(|| {
            error!(
                monitor_id = monitor_id,
                "Monitor with specified ID not found"
            );
            WallpaperError::MonitorNotFound(monitor_id.to_string())
        })?;

        if let Err(error) = self.set_wallpaper(monitor, image_path) {
            match error {
                WallpaperError::SetWallpaper(_) => {
                    self.retry_set_wallpaper(monitor_id, image_path)?
                }
                _ => {
                    error!(
                        error = %error,
                        "Failed to set wallpaper for monitor"
                    );
                    return Err(error.into());
                }
            }
        }

        Ok(())
    }

    /// Gets all available monitors with caching
    fn list_available_monitors(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        self.monitor_provider.get_monitors()
    }

    /// Forces a refresh of monitor information
    fn reload_monitor_configuration(&self) -> DwallResult<HashMap<String, DisplayMonitor>> {
        self.monitor_provider.refresh_monitors()
    }

    /// Sets lock screen wallpaper
    fn set_lock_screen_image_internal(image_path: &Path) -> DwallResult<()> {
        let image_path_hstring = HSTRING::from(image_path);
        let uri = Uri::CreateUri(&image_path_hstring).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to create URI for lock screen image"
            );
            e
        })?;

        debug!(path = ?uri.DisplayUri(), "Target lock screen image");

        let current_lock_screen_image_uri = Self::get_current_lock_screen_image()?;

        if uri.Equals(&current_lock_screen_image_uri)? {
            info!(path = %image_path.display(), "Lock screen image already set");
            return Ok(());
        }

        let file = StorageFile::GetFileFromPathAsync(&image_path_hstring).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to get storage file for lock screen"
            );
            e
        })?;

        let file = wait_bounded(&file, LOCK_SCREEN_TIMEOUT).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to retrieve async storage file"
            );
            e
        })?;

        let i_storage_file: IStorageFile = file.cast().map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to cast storage file"
            );
            e
        })?;

        let result = LockScreen::SetImageFileAsync(&i_storage_file).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to set lock screen image async"
            );
            e
        })?;

        wait_action_bounded(&result, LOCK_SCREEN_TIMEOUT).map_err(|e| {
            error!(
                path = %image_path.display(),
                error = %e,
                "Failed to complete lock screen image setting"
            );
            e
        })?;

        info!(path = %image_path.display(), "Lock screen image updated");
        Ok(())
    }

    fn retry_set_wallpaper(&self, monitor_id: &str, wallpaper_path: &Path) -> DwallResult<()> {
        warn!("Refreshing monitor information and retrying...");
        self.reload_monitor_configuration()?;

        let monitors = self.list_available_monitors()?;
        let monitor = monitors.get(monitor_id).ok_or_else(|| {
            error!(
                monitor_id = monitor_id,
                "Monitor with specified ID not found"
            );
            WallpaperError::MonitorNotFound(monitor_id.to_string())
        })?;

        self.set_wallpaper(monitor, wallpaper_path).map_err(|e| {
            error!(
                error = %e,
                monitor_id = monitor_id,
                wallpaper_path = %wallpaper_path.display(),
                "Failed to set wallpaper for monitor after refresh"
            );
            e
        })?;

        Ok(())
    }

    fn has_wallpaper(
        &self,
        monitor_path: &HSTRING,
        wallpaper_path: &HSTRING,
    ) -> WallpaperResult<bool> {
        debug!(
            monitor_path = %monitor_path,
            "Checking existing wallpaper for monitor"
        );

        let current_wallpaper_ptr =
            match unsafe { self.desktop_wallpaper.GetWallpaper(monitor_path) } {
                Ok(path) => path,
                Err(e) => {
                    // Treat "unknown" as "not the requested wallpaper" so a transient
                    // COM failure does not stop the cycle from applying it.
                    warn!(
                        error = %e,
                        monitor_path = %monitor_path,
                        "Failed to read the current wallpaper; applying anyway"
                    );
                    return Ok(false);
                }
            };

        let current_wallpaper_path = unsafe {
            // SAFETY: `GetWallpaper` returned an owned `LPWSTR`; the guard frees it
            // with `CoTaskMemFree` once `to_hstring` has copied the contents.
            let owned = OwnedCoTaskMemString(current_wallpaper_ptr);
            owned.0.to_hstring()
        };
        let is_same = *wallpaper_path == current_wallpaper_path;

        debug!(
            current = %current_wallpaper_path,
            new = %wallpaper_path,
            is_same = is_same,
            "Wallpaper comparison"
        );

        Ok(is_same)
    }

    fn set_wallpaper(
        &self,
        monitor: &DisplayMonitor,
        wallpaper_path: &Path,
    ) -> WallpaperResult<()> {
        let wallpaper_path = HSTRING::from(wallpaper_path);
        let device_path = HSTRING::from(monitor.device_path());

        if self.has_wallpaper(&device_path, &wallpaper_path)? {
            debug!(
                monitor_id = monitor.device_path(),
                wallpaper_path = %wallpaper_path,
                "Wallpaper already set for monitor, skipping"
            );
            return Ok(());
        };

        let applied = unsafe {
            self.desktop_wallpaper
                .SetWallpaper(&device_path, &wallpaper_path)
        };

        match applied {
            Ok(()) => {
                info!(
                    monitor_id = monitor.device_path(),
                    wallpaper_path = %wallpaper_path,
                    "Applied wallpaper"
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    error = %e,
                    monitor_id = monitor.device_path(),
                    wallpaper_path = %wallpaper_path,
                    "Failed to set wallpaper for monitor"
                );
                Err(WallpaperError::SetWallpaper(e))
            }
        }
    }

    fn get_current_lock_screen_image() -> DwallResult<Uri> {
        let result = LockScreen::OriginalImageFile().map_err(|e| {
            error!(error = %e, "Failed to retrieve lock screen image");
            e
        })?;

        debug!(path = ?result.DisplayUri(), "Current lock screen image");
        Ok(result)
    }
}

impl WallpaperProvider for WallpaperSetter {
    fn set_monitor_wallpaper(&self, monitor_id: &str, image_path: &Path) -> DwallResult<()> {
        self.set_monitor_wallpaper_internal(monitor_id, image_path)
    }

    fn set_lock_screen_image(&self, image_path: &Path) -> DwallResult<()> {
        Self::set_lock_screen_image_internal(image_path)
    }
}

impl Drop for WallpaperSetter {
    fn drop(&mut self) {
        if self.should_cleanup_com {
            unsafe {
                CoUninitialize();
            }
            debug!("COM resources released");
        }
    }
}
