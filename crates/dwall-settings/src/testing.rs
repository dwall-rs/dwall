//! Test-only scratch directories.
//!
//! A test that needs files on disk used to build a fixed path under the temporary
//! directory and remove it by hand at the end. Two tests with the same name then
//! shared one directory, and a failing assertion left its files behind for every
//! later run. This guard removes the directory when it goes out of scope, panic
//! included, and the name carries a process-wide counter so two tests cannot
//! collide even when they run in parallel.

use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A uniquely named temporary directory, removed when the guard is dropped.
pub(crate) struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    /// Creates a fresh scratch directory whose name is unique per call.
    pub(crate) fn new(name: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("dwall_{name}_{}_{unique}", std::process::id()));

        // A leftover directory from a killed run must not be reused.
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory under the temporary directory");

        Self { path }
    }
}

impl Deref for ScratchDir {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for ScratchDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
