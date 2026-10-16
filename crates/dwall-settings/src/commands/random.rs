//! Random-mode status commands.

use serde::Serialize;

use crate::error::DwallSettingsResult;

/// The theme drawn for a given local date in random mode.
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
#[cfg_attr(feature = "typegen", ts(export))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RandomSelection {
    /// Local date (`YYYY-MM-DD`) the theme was drawn for.
    pub date: String,
    pub theme_id: String,
}

/// Today's random-mode selection, or `None` when the daemon isn't running.
#[tauri::command]
pub fn get_random_selection() -> DwallSettingsResult<Option<RandomSelection>> {
    // The state file is written by the daemon; without one running it is stale.
    if !crate::daemon::is_daemon_running()? {
        return Ok(None);
    }

    Ok(dwall::random_state::read()
        .unwrap_or(None)
        .map(|selection| RandomSelection {
            date: selection.date,
            theme_id: selection.theme_id,
        }))
}
