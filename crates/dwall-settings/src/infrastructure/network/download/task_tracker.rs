//! Download task tracking (pure concurrency control, no business logic)

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

use tokio::sync::Mutex;

use crate::error::DwallSettingsResult;

/// Download task information
#[derive(Debug)]
pub(super) struct DownloadTask {
    pub cancel: Arc<AtomicBool>,
}

/// Manages download tasks and their cancellation flags
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
    pub async fn add_task(&self, task_id: &str) -> DwallSettingsResult<Arc<AtomicBool>> {
        let mut tasks = self.download_tasks.lock().await;
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
        drop(tasks);

        Ok(cancel_flag)
    }

    /// Remove a download task
    pub async fn remove_task(&self, task_id: &str) {
        let mut tasks = self.download_tasks.lock().await;
        tasks.remove(task_id);
    }

    // /// Check if a task should be cancelled
    // pub fn is_cancelled(&self, cancel_flag: &Arc<AtomicBool>) -> bool {
    //     cancel_flag.load(Ordering::Relaxed)
    // }

    /// Cancel a download task
    pub async fn cancel_task(&self, task_id: &str) {
        let tasks = self.download_tasks.lock().await;

        if let Some(task) = tasks.get(task_id) {
            task.cancel.store(true, Ordering::Relaxed);
            info!("Requested cancellation of task '{task_id}'");
        } else {
            warn!("Attempted to cancel non-existent task '{task_id}'");
        }
    }
}
