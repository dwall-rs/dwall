//! Persisted daily random-theme selection (`random_selection.json`).
//!
//! The daemon's random-mode pick lives in memory; this small file exposes it to
//! the settings UI so it can show which theme is active today.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::DwallResult;
use crate::lazy::DWALL_CONFIG_DIR;

const FILE_NAME: &str = "random_selection.json";

/// The theme drawn for a given local date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomSelection {
    /// Local date the theme was drawn for, formatted `YYYY-MM-DD`.
    pub date: String,
    pub theme_id: String,
}

impl RandomSelection {
    pub fn new(date: &time::Date, theme_id: String) -> Self {
        Self {
            date: date_key(date),
            theme_id,
        }
    }
}

/// Formats a local date as the `YYYY-MM-DD` key stored in the file.
pub fn date_key(date: &time::Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        date.month().as_u8(),
        date.day()
    )
}

/// Path of the state file inside `directory`.
fn path_in(directory: &Path) -> PathBuf {
    directory.join(FILE_NAME)
}

/// Reads the persisted selection, if any.
pub fn read() -> DwallResult<Option<RandomSelection>> {
    read_from(&DWALL_CONFIG_DIR)
}

/// Reads the persisted selection from `directory`, if any.
pub fn read_from(directory: &Path) -> DwallResult<Option<RandomSelection>> {
    let path = path_in(directory);
    if !path.is_file() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&content)?))
}

/// Persists the selection.
pub fn write(selection: &RandomSelection) -> DwallResult<()> {
    write_to(&DWALL_CONFIG_DIR, selection)
}

/// Persists the selection in `directory`, creating it when needed.
///
/// Writes through a sibling temporary file and renames it into place. A crash in
/// the middle of `fs::write` used to leave a truncated JSON document, and the next
/// start could only discard it and draw a different theme.
pub fn write_to(directory: &Path, selection: &RandomSelection) -> DwallResult<()> {
    let content = serde_json::to_string_pretty(selection)?;
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
    use time::{Date, Month};

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_random_state_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// The key is compared as a string by the settings UI, so single-digit months
    /// and days have to be padded.
    #[test]
    fn date_key_pads_single_digit_months_and_days() {
        assert_eq!(date_key(&Date::new(2026, Month::January, 5)), "2026-01-05");
        assert_eq!(
            date_key(&Date::new(2026, Month::December, 31)),
            "2026-12-31"
        );
        assert_eq!(date_key(&Date::new(999, Month::September, 9)), "0999-09-09");
    }

    #[test]
    fn a_selection_is_built_from_the_date_key() {
        let selection = RandomSelection::new(&Date::new(2026, Month::June, 21), "Big Sur".into());

        assert_eq!(selection.date, "2026-06-21");
        assert_eq!(selection.theme_id, "Big Sur");
    }

    #[test]
    fn a_missing_file_has_no_selection() {
        let dir = scratch("missing");

        assert!(read_from(&dir).unwrap().is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_selection_round_trips_through_the_file() {
        let dir = scratch("roundtrip");
        let selection = RandomSelection::new(&Date::new(2026, Month::June, 21), "Big Sur".into());

        write_to(&dir, &selection).unwrap();
        let read_back = read_from(&dir).unwrap().expect("the file was just written");

        assert_eq!(read_back.date, selection.date);
        assert_eq!(read_back.theme_id, selection.theme_id);

        // It lands next to the configuration, not in the cache directory.
        assert!(dir.join("random_selection.json").is_file());
        assert!(
            !dir.join("random_selection.json.tmp").exists(),
            "the temporary file is renamed away"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// A truncated file is an error the caller can log, not a silently empty state
    /// that would make the daemon draw a second theme for the same day.
    #[test]
    fn corrupt_selection_json_is_an_error() {
        let dir = scratch("corrupt");
        fs::write(dir.join("random_selection.json"), "{ not json").unwrap();

        assert!(read_from(&dir).is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    /// The settings UI reads this file, so the field names are an interface; the
    /// snapshot fails if either is renamed.
    #[test]
    fn snapshot_random_selection() {
        insta::assert_json_snapshot!(
            "random_selection",
            RandomSelection::new(&Date::new(2026, Month::June, 21), "Big Sur".to_string())
        );
    }

    #[test]
    fn writing_creates_the_directory() {
        let dir = scratch("creates").join("nested");

        write_to(
            &dir,
            &RandomSelection::new(&Date::new(2026, Month::June, 21), "Big Sur".into()),
        )
        .unwrap();

        assert!(read_from(&dir).unwrap().is_some());

        let _ = fs::remove_dir_all(dir.parent().unwrap());
    }
}
