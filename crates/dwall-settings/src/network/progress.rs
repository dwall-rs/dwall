//! Download progress reporting.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use tauri::{Emitter, Runtime, WebviewWindow};

/// Download progress tracking
#[derive(Serialize, Clone, Debug)]
pub struct DownloadProgress<'a> {
    pub task_id: &'a str,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Progress notification service for a borrowed window handle.
pub struct ProgressEmitter<'a, R: Runtime> {
    window: &'a WebviewWindow<R>,
}

impl<'a, R: Runtime> ProgressEmitter<'a, R> {
    pub fn new(window: &'a WebviewWindow<R>) -> Self {
        Self { window }
    }

    /// The window this emitter reports to, for building owned callbacks.
    pub fn window(&self) -> &WebviewWindow<R> {
        self.window
    }
}

/// Progress notification service that owns its window handle.
///
/// Owning the handle is what allows a `'static` download callback to be built
/// from it.
pub struct OwnedProgressEmitter<R: Runtime> {
    window: WebviewWindow<R>,
}

impl<R: Runtime + 'static> OwnedProgressEmitter<R> {
    pub fn new(window: &WebviewWindow<R>) -> Self {
        Self {
            window: window.clone(),
        }
    }

    pub fn emit_progress(&self, progress: DownloadProgress) -> Result<(), tauri::Error> {
        self.window.emit("download-progress", progress)
    }

    /// Builds a download callback that reports at most once per percent.
    ///
    /// Emitting per chunk would mean thousands of IPC messages for a large theme
    /// archive.
    pub fn throttled_callback(self) -> Arc<dyn Fn(u64, u64) + Send + Sync> {
        let last_percent = AtomicU64::new(u64::MAX);

        Arc::new(move |written: u64, total: u64| {
            if total == 0 {
                return;
            }

            let percent = written.saturating_mul(100) / total;
            if written < total && last_percent.swap(percent, Ordering::Relaxed) == percent {
                return;
            }

            let _ = self.emit_progress(DownloadProgress {
                task_id: "theme",
                downloaded_bytes: written,
                total_bytes: total,
            });
        })
    }
}
