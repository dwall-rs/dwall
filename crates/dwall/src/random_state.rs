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
pub fn write_to(directory: &Path, selection: &RandomSelection) -> DwallResult<()> {
    let content = serde_json::to_string_pretty(selection)?;
    fs::create_dir_all(directory)?;
    fs::write(path_in(directory), content)?;
    Ok(())
}
