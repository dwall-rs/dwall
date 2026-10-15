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
