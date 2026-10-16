//! Auto-start toggle, backed by the shared implementation in `dwall`.
//!
//! The registry work lives in `dwall::install_autostart` and friends so the daemon
//! (`dwall.exe --install-autostart`) and this toggle cannot disagree about the
//! command-line quoting or the "is it registered?" comparison.

use std::path::Path;

use crate::error::DwallSettingsResult;

/// Manages the Windows registry auto-start entry for the bundled daemon.
pub struct AutoStartProvider;

impl AutoStartProvider {
    /// The daemon this settings app installs and starts.
    fn executable() -> DwallSettingsResult<&'static Path> {
        crate::daemon::daemon_path()
    }

    /// Enables auto-start by adding the daemon to the registry
    pub fn enable() -> DwallSettingsResult<()> {
        dwall::install_autostart(Self::executable()?)?;
        Ok(())
    }

    /// Disables auto-start by removing the daemon from the registry
    pub fn disable() -> DwallSettingsResult<()> {
        dwall::uninstall_autostart()?;
        Ok(())
    }

    /// Checks if auto-start is currently enabled
    pub fn is_enabled() -> DwallSettingsResult<bool> {
        Ok(dwall::is_autostart_installed(Self::executable()?)?)
    }
}
