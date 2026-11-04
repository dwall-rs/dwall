//! Persisted colour-scheme pause (`color_scheme_pause.json`).
//!
//! The daemon switches the Windows theme from the sun's position, so a change the user
//! makes by hand is undone on the next cycle - at most one interval later. This file is
//! the truce: while it says so, the daemon leaves the theme alone. The settings window
//! writes it when the user asks for a pause, and the daemon writes it itself when it
//! notices the theme changed without it.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::DwallResult;
use crate::lazy::DWALL_CONFIG_DIR;

const FILE_NAME: &str = "color_scheme_pause.json";

/// How long a pause lasts, whether the daemon or the user started it.
///
/// Long enough to cover the manual override that prompts it, short enough that the sun
/// takes over again without the user having to remember to resume.
pub const PAUSE_SECONDS: u64 = 3600;

/// Who asked for the pause.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PauseCause {
    /// The user asked for it from the settings window.
    Manual,
    /// The daemon saw the theme change without having changed it itself.
    External,
}

/// A window during which the daemon leaves the colour scheme alone.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorSchemePause {
    /// Unix seconds at which automatic switching resumes.
    pub until: u64,
    /// Who asked for the pause, so the window can say why it is paused.
    pub cause: PauseCause,
}

impl ColorSchemePause {
    /// A pause that ends `seconds` after `now`.
    pub fn for_seconds(now: u64, seconds: u64, cause: PauseCause) -> Self {
        Self {
            until: now.saturating_add(seconds),
            cause,
        }
    }

    /// Whether the pause still holds at `now`.
    pub fn is_active(&self, now: u64) -> bool {
        now < self.until
    }

    /// Seconds left at `now`, zero once the pause has run out.
    pub fn remaining(&self, now: u64) -> u64 {
        self.until.saturating_sub(now)
    }
}

/// Path of the state file inside `directory`.
fn path_in(directory: &Path) -> PathBuf {
    directory.join(FILE_NAME)
}

/// Reads the pause from the configuration directory, if there is one.
pub fn read() -> DwallResult<Option<ColorSchemePause>> {
    read_from(&DWALL_CONFIG_DIR)
}

/// Reads the pause from `directory`, if there is one.
///
/// A file that cannot be parsed is removed rather than reported: it is read on every
/// cycle, a warning per cycle would drown the log, and the only thing the file can mean
/// is "do not switch" - dropping it resumes switching, which is the safe direction.
pub fn read_from(directory: &Path) -> DwallResult<Option<ColorSchemePause>> {
    let path = path_in(directory);
    if !path.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)?;
    match serde_json::from_str(&content) {
        Ok(pause) => Ok(Some(pause)),
        Err(e) => {
            warn!(
                path = %path.display(),
                error = %e,
                "Discarding an unreadable colour-scheme pause"
            );
            let _ = fs::remove_file(&path);

            Ok(None)
        }
    }
}

/// Persists a pause in the configuration directory.
pub fn write(pause: &ColorSchemePause) -> DwallResult<()> {
    write_to(&DWALL_CONFIG_DIR, pause)
}

/// Removes the pause from the configuration directory, if there is one.
pub fn clear() -> DwallResult<()> {
    clear_from(&DWALL_CONFIG_DIR)
}

/// Removes the pause from `directory`, if there is one.
pub fn clear_from(directory: &Path) -> DwallResult<()> {
    match fs::remove_file(path_in(directory)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

/// Persists a pause in `directory`, creating it when needed.
///
/// Writes through a sibling temporary file and renames it into place: the daemon reads
/// this file on every cycle, and a half-written document would be discarded as corrupt.
pub fn write_to(directory: &Path, pause: &ColorSchemePause) -> DwallResult<()> {
    let content = serde_json::to_string_pretty(pause)?;
    fs::create_dir_all(directory)?;

    let path = path_in(directory);
    let temporary = path.with_extension("json.tmp");

    fs::write(&temporary, content)?;

    if let Err(e) = fs::rename(&temporary, &path) {
        let _ = fs::remove_file(&temporary);
        return Err(e.into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_pause_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn a_pause_ends_the_configured_number_of_seconds_later() {
        let pause = ColorSchemePause::for_seconds(1_000, 3_600, PauseCause::Manual);

        assert_eq!(pause.until, 4_600);
        assert!(pause.is_active(4_599));
        // The boundary belongs to switching: at `until` the sun takes over again.
        assert!(!pause.is_active(4_600));
        assert_eq!(pause.remaining(1_000), 3_600);
        assert_eq!(pause.remaining(5_000), 0);
    }

    /// A clock that jumps backwards must not leave a pause active forever.
    #[test]
    fn the_remaining_time_never_underflows() {
        let pause = ColorSchemePause::for_seconds(u64::MAX - 10, 60, PauseCause::External);

        assert_eq!(pause.until, u64::MAX);
        assert_eq!(pause.remaining(0), u64::MAX);
    }

    #[test]
    fn a_missing_file_is_no_pause() {
        let dir = scratch("missing");

        assert!(read_from(&dir).unwrap().is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_pause_round_trips_through_the_file() {
        let dir = scratch("roundtrip");
        let pause = ColorSchemePause::for_seconds(1_000, 60, PauseCause::External);

        write_to(&dir, &pause).unwrap();
        let read_back = read_from(&dir).unwrap().expect("the file was just written");

        assert_eq!(read_back, pause);
        assert!(dir.join("color_scheme_pause.json").is_file());
        assert!(
            !dir.join("color_scheme_pause.json.tmp").exists(),
            "the temporary file is renamed away"
        );

        clear_from(&dir).unwrap();
        assert!(read_from(&dir).unwrap().is_none());
        // Clearing twice is what a resumed cycle and a settings click can do.
        clear_from(&dir).unwrap();

        let _ = fs::remove_dir_all(&dir);
    }

    /// The daemon reads this file every cycle; a file it cannot parse must not turn
    /// into a warning per cycle, and resuming switching is the safe reading.
    #[test]
    fn an_unreadable_pause_is_discarded() {
        let dir = scratch("corrupt");
        fs::write(dir.join("color_scheme_pause.json"), "{ not json").unwrap();

        assert!(read_from(&dir).unwrap().is_none());
        assert!(
            !dir.join("color_scheme_pause.json").exists(),
            "the file is removed so the next cycle starts clean"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// The settings window parses this document, so the field names and the cause
    /// spelling are an interface.
    #[test]
    fn snapshot_color_scheme_pause() {
        insta::assert_json_snapshot!(
            "color_scheme_pause",
            ColorSchemePause::for_seconds(1_000, 3_600, PauseCause::External)
        );
    }
}
