//! Colour-scheme pause commands.
//!
//! The pause lives in a file next to the configuration because both binaries need it:
//! the window writes it, and the daemon reads it on every cycle.

use dwall::color_scheme_pause::{self, ColorSchemePause, PauseCause};

use crate::error::{DwallSettingsError, DwallSettingsResult};

/// The pause in force, if any.
///
/// The window polls this while it is open: a pause can also start without it, when the
/// daemon notices a theme change it did not make.
#[tauri::command]
pub async fn get_color_scheme_pause() -> DwallSettingsResult<Option<ColorSchemePause>> {
    Ok(color_scheme_pause::read()?)
}

/// Pauses automatic switching for `minutes`, or resumes it when `minutes` is zero.
///
/// Returns the pause it just wrote, so the window can show the countdown without a
/// second round trip.
#[tauri::command]
pub async fn set_color_scheme_pause(minutes: u32) -> DwallSettingsResult<Option<ColorSchemePause>> {
    if minutes == 0 {
        color_scheme_pause::clear()?;
        return Ok(None);
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| DwallSettingsError::Other(e.to_string()))?
        .as_secs();

    let pause = ColorSchemePause::for_seconds(now, u64::from(minutes) * 60, PauseCause::Manual);
    color_scheme_pause::write(&pause)?;

    Ok(Some(pause))
}
