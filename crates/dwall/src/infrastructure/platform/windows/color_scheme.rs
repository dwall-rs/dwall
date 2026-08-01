//! Windows implementation of the ColorSchemeProvider trait

use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Registry::{KEY_QUERY_VALUE, KEY_SET_VALUE, REG_DWORD},
    UI::WindowsAndMessaging::{HWND_BROADCAST, SendNotifyMessageW, WM_SETTINGCHANGE},
};

use crate::{
    DwallResult, RegistryKey,
    domain::visual::{ColorScheme, ColorSchemeProvider},
    utils::string::WideStringExt,
};

/// Color scheme manager for Windows system theme management
pub struct ColorSchemeScheduler;

impl ColorSchemeScheduler {
    const PERSONALIZE_KEY_PATH: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
    const APPS_THEME_VALUE: &str = "AppsUseLightTheme";
    const SYSTEM_THEME_VALUE: &str = "SystemUsesLightTheme";

    pub fn new() -> Self {
        Self
    }
}

impl Default for ColorSchemeScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorSchemeProvider for ColorSchemeScheduler {
    /// Retrieve the current system color scheme from registry
    fn get_current_scheme(&self) -> DwallResult<ColorScheme> {
        debug!("Retrieving current system color scheme");
        let registry_key = RegistryKey::open(Self::PERSONALIZE_KEY_PATH, KEY_QUERY_VALUE)?;

        let mut data: u32 = 0;
        let mut data_size = std::mem::size_of_val(&data) as u32;
        let mut data_type = REG_DWORD;

        registry_key.query(
            Self::APPS_THEME_VALUE,
            Some(std::ptr::addr_of_mut!(data_type)),
            Some(std::ptr::addr_of_mut!(data) as *mut u8),
            Some(&mut data_size),
        )?;

        debug!(value = data, "Retrieved app theme value from registry");

        let scheme = if data == 1 {
            debug!("Current color scheme is Light");
            ColorScheme::Light
        } else {
            debug!("Current color scheme is Dark");
            ColorScheme::Dark
        };

        Ok(scheme)
    }

    fn set_color_scheme(&self, scheme: ColorScheme) -> DwallResult<()> {
        let current_color_scheme = self.get_current_scheme()?;
        if current_color_scheme == scheme {
            info!(scheme = %scheme, "Color scheme is already set");
            return Ok(());
        }

        info!(scheme = %scheme, "Setting system color scheme");
        let registry_key = RegistryKey::open(Self::PERSONALIZE_KEY_PATH, KEY_SET_VALUE)?;

        let value = scheme.to_le_bytes();

        registry_key
            .set(Self::APPS_THEME_VALUE, REG_DWORD, &value)
            .map_err(|e| {
                error!(error = ?e, "Failed to set apps theme value");
                e
            })?;
        info!(scheme = %scheme, "Successfully set apps theme value");

        registry_key
            .set(Self::SYSTEM_THEME_VALUE, REG_DWORD, &value)
            .map_err(|e| {
                error!(error = ?e, "Failed to set system theme value");
                e
            })?;
        info!(scheme = %scheme, "Successfully set system theme value");

        notify_theme_change()?;

        let current_color_scheme = self.get_current_scheme()?;
        if !verify_theme_change(&scheme, &current_color_scheme)? {
            warn!("Theme change may not have been applied correctly");
        }

        Ok(())
    }
}

fn verify_theme_change(expected: &ColorScheme, current: &ColorScheme) -> DwallResult<bool> {
    std::thread::sleep(std::time::Duration::from_millis(100));
    Ok(current == expected)
}

/// Notify the system about theme changes
fn notify_theme_change() -> DwallResult<()> {
    trace!("Broadcasting theme change notifications");

    let lparam = Vec::from_str("ImmersiveColorSet");

    unsafe {
        SendNotifyMessageW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(lparam.as_ptr() as isize),
        )?;
    }

    debug!("Notified system about theme change");

    Ok(())
}
