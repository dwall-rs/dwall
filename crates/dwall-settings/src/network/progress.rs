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
    /// `task_id` identifies the download to the window (the theme id today), so a
    /// progress message can be attributed instead of being assumed to belong to
    /// whichever download happens to be running. Emitting per chunk would mean
    /// thousands of IPC messages for a large theme archive.
    pub fn throttled_callback(
        self,
        task_id: impl Into<String>,
    ) -> Arc<dyn Fn(u64, u64) + Send + Sync> {
        let last_percent = AtomicU64::new(NO_PERCENT_YET);
        let task_id = task_id.into();

        Arc::new(move |written: u64, total: u64| {
            let previous = last_percent.load(Ordering::Relaxed);
            let Some(percent) = percent_to_emit(written, total, previous) else {
                return;
            };
            last_percent.store(percent, Ordering::Relaxed);

            let _ = self.emit_progress(DownloadProgress {
                task_id: &task_id,
                downloaded_bytes: written,
                total_bytes: total,
            });
        })
    }
}

/// The percent to report for a `written`/`total` update, or `None` when the update
/// repeats what was already shown.
///
/// Split out of the callback because the throttling is the part worth testing and it
/// needs no window: a `WebviewWindow` cannot be built in a test.
fn percent_to_emit(written: u64, total: u64, previous_percent: u64) -> Option<u64> {
    if total == 0 {
        // A mirror or a proxy may answer without `Content-Length`; there is no
        // percentage to show.
        return None;
    }

    // A server that sends more than it announced still means "finished", and this
    // keeps the arithmetic away from the overflow that used to produce a percentage
    // in the quintillions for such a response.
    if written >= total {
        return Some(100);
    }

    let percent = written.saturating_mul(100) / total;

    // The final chunk is always reported, even when it lands on the percent that was
    // already shown, so the UI always ends at `written == total`. That case is the
    // branch above; here the update is throttled.
    if previous_percent == percent {
        return None;
    }

    Some(percent)
}

/// No percent has been reported yet.
const NO_PERCENT_YET: u64 = u64::MAX;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_progress_without_a_total_size() {
        assert_eq!(percent_to_emit(0, 0, NO_PERCENT_YET), None);
        assert_eq!(percent_to_emit(1024, 0, NO_PERCENT_YET), None);
    }

    #[test]
    fn one_update_per_percent() {
        // The first update of a percent is reported…
        assert_eq!(percent_to_emit(10, 100, NO_PERCENT_YET), Some(10));
        // …and a second chunk inside it is not.
        assert_eq!(percent_to_emit(10, 100, 10), None);
        // The next percent is.
        assert_eq!(percent_to_emit(20, 100, 10), Some(20));
    }

    /// Without this the bar could stop at 99%: the last chunk often lands in a
    /// percent that was already reported.
    #[test]
    fn the_final_chunk_is_always_reported() {
        assert_eq!(percent_to_emit(100, 100, 100), Some(100));
        assert_eq!(percent_to_emit(100, 100, NO_PERCENT_YET), Some(100));
    }

    #[test]
    fn a_written_count_beyond_the_total_still_reports_a_hundred() {
        assert_eq!(percent_to_emit(u64::MAX, 7, NO_PERCENT_YET), Some(100));
        assert_eq!(percent_to_emit(u64::MAX, u64::MAX, 100), Some(100));
    }

    #[test]
    fn progress_is_reported_from_the_first_byte() {
        assert_eq!(percent_to_emit(1, 1_000_000, NO_PERCENT_YET), Some(0));
        // The same percent is not repeated: one message per percent, and the first
        // byte already showed 0%.
        assert_eq!(percent_to_emit(5_000, 1_000_000, 0), None);
        assert_eq!(percent_to_emit(20_000, 1_000_000, 0), Some(2));
    }
}
