//! Download task tracking and cancellation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use crate::error::DwallSettingsResult;

/// Download task information
#[derive(Debug)]
struct DownloadTask {
    cancel: Arc<AtomicBool>,
}

/// Manages download tasks and cancellation flags
///
/// The critical sections are tiny map operations, so a `std::sync::Mutex` is used
/// and the methods stay synchronous: that lets [`remove_task`](Self::remove_task)
/// run from a `Drop` guard, which is what keeps a failed download from leaving its
/// theme permanently marked as "already being processed".
pub struct DownloadTaskManager {
    download_tasks: LazyLock<Arc<Mutex<HashMap<String, DownloadTask>>>>,
}

impl DownloadTaskManager {
    /// Create a new download task manager
    pub fn new() -> Self {
        Self {
            download_tasks: LazyLock::new(|| Arc::new(Mutex::new(HashMap::new()))),
        }
    }

    /// Add a new download task and return its cancellation flag
    pub fn add_task(&self, task_id: &str) -> DwallSettingsResult<Arc<AtomicBool>> {
        let mut tasks = self.lock();

        if tasks.contains_key(task_id) {
            return Err(crate::error::DwallSettingsError::Other(format!(
                "Task '{task_id}' is already being processed"
            )));
        }

        let cancel_flag = Arc::new(AtomicBool::new(false));
        tasks.insert(
            task_id.to_string(),
            DownloadTask {
                cancel: cancel_flag.clone(),
            },
        );

        Ok(cancel_flag)
    }

    /// Remove a download task
    pub fn remove_task(&self, task_id: &str) {
        self.lock().remove(task_id);
    }

    /// Cancel a download task
    pub fn cancel_task(&self, task_id: &str) {
        let tasks = self.lock();

        if let Some(task) = tasks.get(task_id) {
            task.cancel.store(true, Ordering::Relaxed);
            info!("Requested cancellation of task '{task_id}'");
        } else {
            warn!("Attempted to cancel non-existent task '{task_id}'");
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, DownloadTask>> {
        self.download_tasks
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for DownloadTaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A task is identified by the theme id, so a second download of the same theme
    /// has to be refused rather than racing the first one.
    #[test]
    fn a_duplicate_task_is_refused() {
        let manager = DownloadTaskManager::new();
        manager.add_task("Big Sur").unwrap();

        let error = manager
            .add_task("Big Sur")
            .expect_err("already downloading");

        assert!(error.to_string().contains("Big Sur"), "{error}");
    }

    #[test]
    fn another_task_can_be_added_at_the_same_time() {
        let manager = DownloadTaskManager::new();
        manager.add_task("Big Sur").unwrap();

        assert!(manager.add_task("Catalina").is_ok());
    }

    #[test]
    fn cancelling_a_task_sets_its_flag() {
        let manager = DownloadTaskManager::new();
        let cancel = manager.add_task("Big Sur").unwrap();

        assert!(!cancel.load(Ordering::Relaxed));

        manager.cancel_task("Big Sur");

        assert!(cancel.load(Ordering::Relaxed));
    }

    /// Cancelling from the UI can race the download finishing; that is a warning,
    /// not a panic.
    #[test]
    fn cancelling_an_unknown_task_is_a_noop() {
        let manager = DownloadTaskManager::new();

        manager.cancel_task("never started");

        assert!(manager.add_task("never started").is_ok());
    }

    /// The guard removes the task when a download ends, so the same theme can be
    /// downloaded again — including after a failure.
    #[test]
    fn removing_a_task_lets_it_be_added_again() {
        let manager = DownloadTaskManager::new();
        let first = manager.add_task("Big Sur").unwrap();

        manager.remove_task("Big Sur");

        let second = manager.add_task("Big Sur").expect("the task is gone");
        assert!(
            !second.load(Ordering::Relaxed),
            "a new task gets a fresh flag"
        );

        // The removed task's flag is not the one the manager hands out any more.
        manager.cancel_task("Big Sur");
        assert!(!first.load(Ordering::Relaxed));
        assert!(second.load(Ordering::Relaxed));
    }
}
