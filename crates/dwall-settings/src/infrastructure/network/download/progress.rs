//! Download progress reporting (pure technical)

use serde::Serialize;
use tauri::{Emitter, Runtime, WebviewWindow};

/// Download progress tracking
#[derive(Serialize, Clone, Debug)]
pub struct DownloadProgress<'a> {
    pub task_id: &'a str,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Progress notification service
pub struct ProgressEmitter<'a, R: Runtime> {
    window: &'a WebviewWindow<R>,
}

impl<'a, R: Runtime> ProgressEmitter<'a, R> {
    pub fn new(window: &'a WebviewWindow<R>) -> Self {
        Self { window }
    }

    pub fn emit_progress(&self, progress: DownloadProgress) -> Result<(), tauri::Error> {
        self.window.emit("download-progress", progress)
    }
}
