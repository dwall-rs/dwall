//! Configuration file change detection

use std::{fs, path::PathBuf, time::SystemTime};

use crate::error::DwallResult;

/// Watches configuration file for changes
pub(crate) struct ConfigWatcher {
    config_path: PathBuf,
    last_modified: Option<SystemTime>,
}

impl ConfigWatcher {
    /// Creates a new ConfigWatcher for the specified config path
    pub(crate) fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            last_modified: None,
        }
    }

    /// Returns the config path being watched
    pub(crate) fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Gets the current modification time from the filesystem
    fn get_file_modified_time(&self) -> DwallResult<Option<SystemTime>> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(&self.config_path)?;
        let modified = metadata.modified()?;
        Ok(Some(modified))
    }

    /// Checks if the configuration file has changed since last check
    ///
    /// Returns `true` if:
    /// - The file didn't exist before but now exists
    /// - The file existed before but now doesn't exist
    /// - The file's modification time is different from the last known time
    pub(crate) fn has_changed(&self) -> DwallResult<bool> {
        let current_modified = self.get_file_modified_time()?;

        let has_changed = match (self.last_modified, current_modified) {
            (None, None) => false,
            (Some(_), None) | (None, Some(_)) => true,
            (Some(last), Some(current)) => last != current,
        };

        if has_changed {
            debug!(
                path = %self.config_path.display(),
                last_modified = ?self.last_modified,
                current_modified = ?current_modified,
                "Configuration file change detected"
            );
        }

        Ok(has_changed)
    }

    /// Updates the last known modification time
    pub(crate) fn update_modified_time(&mut self) -> DwallResult<()> {
        self.last_modified = self.get_file_modified_time()?;
        Ok(())
    }
}
