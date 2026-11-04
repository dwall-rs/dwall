//! Random-mode status commands.

use dwall::random_state::RandomSelection;

use crate::error::DwallSettingsResult;

/// Today's random-mode selection, or `None` when the daemon isn't running.
///
/// Answers with the state file's own type: the daemon writes it, this window reads it
/// and the frontend displays it, so a mirror struct here would only be a second copy
/// of the same two fields — and an export that silently overwrote the shared one.
#[tauri::command]
pub fn get_random_selection() -> DwallSettingsResult<Option<RandomSelection>> {
    // The state file is written by the daemon; without one running it is stale.
    if !crate::daemon::is_daemon_running()? {
        return Ok(None);
    }

    match dwall::random_state::read() {
        Ok(selection) => Ok(selection),
        Err(e) => {
            // A corrupt state file means "no selection" to the UI; say why rather than
            // reporting the same None as random mode being off.
            warn!(error = %e, "Could not read the random-mode state");
            Ok(None)
        }
    }
}
