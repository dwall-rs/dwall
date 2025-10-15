//! Download task management
//!
//! This module provides functionality for managing download tasks and tracking progress.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock};

use serde::Serialize;
use tauri::{Emitter, Runtime, WebviewWindow};
use tokio::sync::Mutex;

use crate::error::DwallSettingsResult;

use super::error::DownloadError;

/// Download task information
#[derive(Debug)]
pub(super) struct DownloadTask {
    /// Flag to indicate if the download should be cancelled
    pub cancel: Arc<AtomicBool>,
}

/// Download progress tracking
#[derive(Serialize, Clone, Debug)]
pub(super) struct DownloadProgress<'a> {
    pub theme_id: &'a str,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Progress notification service
pub(super) struct ProgressEmitter<'a, R: Runtime> {
    window: &'a WebviewWindow<R>,
}

impl<'a, R: Runtime> ProgressEmitter<'a, R> {
    pub fn new(window: &'a WebviewWindow<R>) -> Self {
        Self { window }
    }

    pub fn emit_progress(&self, progress: DownloadProgress) -> Result<(), tauri::Error> {
        self.window.emit("download-theme", progress)
    }
}

/// Manages download tasks and their cancellation flags
pub(super) struct DownloadTaskManager {
    download_tasks: LazyLock<Arc<Mutex<HashMap<String, DownloadTask>>>>,
}

impl DownloadTaskManager {
    /// Create a new download task manager
    pub(super) fn new() -> Self {
        Self {
            download_tasks: LazyLock::new(|| Arc::new(Mutex::new(HashMap::new()))),
        }
    }

    /// Add a new download task and return its cancellation flag
    pub(super) async fn add_task(&self, theme_id: &str) -> DwallSettingsResult<Arc<AtomicBool>> {
        let mut tasks = self.download_tasks.lock().await;
        if tasks.contains_key(theme_id) {
            error!(theme_id = theme_id, "Theme is already being downloaded");
            return Err(
                DownloadError::Unknown("Theme is already being downloaded".to_string()).into(),
            );
        }

        // Mark theme as being downloaded with cancel flag
        let cancel_flag = Arc::new(AtomicBool::new(false));
        tasks.insert(
            theme_id.to_string(),
            DownloadTask {
                cancel: cancel_flag.clone(),
            },
        );
        drop(tasks);

        Ok(cancel_flag)
    }

    /// Remove a download task
    pub(super) async fn remove_task(&self, theme_id: &str) {
        let mut tasks = self.download_tasks.lock().await;
        tasks.remove(theme_id);
    }

    /// Check if a task should be cancelled
    pub(super) fn is_cancelled(&self, cancel_flag: &Arc<AtomicBool>) -> bool {
        cancel_flag.load(Ordering::Relaxed)
    }

    /// Cancel a download task
    pub(super) async fn cancel_task(&self, theme_id: &str) {
        let tasks = self.download_tasks.lock().await;

        if let Some(task) = tasks.get(theme_id) {
            // Set the cancel flag to true
            task.cancel.store(true, Ordering::Relaxed);
            info!(
                theme_id = theme_id,
                "Requested cancellation of theme download"
            );
        } else {
            // Theme is not being downloaded
            warn!(
                theme_id = theme_id,
                "Attempted to cancel download for theme that is not being downloaded"
            );
        }

        drop(tasks);
    }
}
