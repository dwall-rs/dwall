//! Lazily initialised per-user directories.
//!
//! These initialisers run on first access from every entry point, including a
//! GUI-subsystem daemon whose panics are invisible (release builds are built with
//! `panic = "abort"` and have no console). A profile directory that cannot be
//! created or determined therefore degrades into a logged error plus a working
//! fallback, never into a process that disappears without a trace; callers that
//! genuinely need the directory fail later with a real error.

use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{env, fs};

/// Creates `path` (and its parents) when missing, logging instead of panicking.
fn ensure_dir(path: &Path) {
    if path.exists() {
        debug!(path = %path.display(), "Directory already exists");
        return;
    }

    match fs::create_dir_all(path) {
        Ok(()) => info!(path = %path.display(), "Directory created successfully"),
        Err(e) => error!(
            error = %e,
            path = %path.display(),
            "Failed to create directory"
        ),
    }
}

/// Base directory for the temporary fallback, reported as an error.
fn temp_base(kind: &str) -> PathBuf {
    let path = env::temp_dir();

    error!(
        kind = kind,
        path = %path.display(),
        "Failed to determine the {kind} directory; falling back to the temporary directory"
    );

    path
}

/// Global configuration directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
/// Falls back to the temporary directory when the user profile cannot be resolved.
pub static DWALL_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::config_dir().unwrap_or_else(|| temp_base("config"));
    let app_config_dir = config_dir.join("dwall");

    ensure_dir(&app_config_dir);

    app_config_dir
});

/// Global cache directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
/// Uses the application bundle identifier for better organization.
pub static DWALL_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let cache_dir = dirs::cache_dir().unwrap_or_else(|| temp_base("cache"));
    let dir = cache_dir.join("com.thep0y.dwall"); // bundle identifier

    ensure_dir(&dir);

    dir
});

/// Global log directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
/// Uses the application bundle identifier for better organization.
pub static DWALL_LOG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let log_dir = DWALL_CACHE_DIR.join("log");

    ensure_dir(&log_dir);

    log_dir
});
