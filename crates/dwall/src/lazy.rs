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

/// Creates `base/name` (and its parents) when missing, logging instead of panicking.
///
/// The base directory is a parameter so this seam can be exercised without touching the
/// real profile; the public statics below are its only production callers.
pub(crate) fn ensure_dir(base: &Path, name: &str) {
    let path = base.join(name);

    if path.exists() {
        debug!(path = %path.display(), "Directory already exists");
        return;
    }

    match fs::create_dir_all(&path) {
        Ok(()) => info!(path = %path.display(), "Directory created successfully"),
        Err(e) => error!(
            error = %e,
            path = %path.display(),
            "Failed to create directory"
        ),
    }
}

/// Reports `base` as the fallback directory for `kind` and returns it.
///
/// The base is a parameter for the same reason as in [`ensure_dir`].
pub(crate) fn temp_base(kind: &str, base: PathBuf) -> PathBuf {
    error!(
        kind = kind,
        path = %base.display(),
        "Failed to determine the {kind} directory; falling back to the temporary directory"
    );

    base
}

/// Global configuration directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
/// Falls back to the temporary directory when the user profile cannot be resolved.
pub static DWALL_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::config_dir().unwrap_or_else(|| temp_base("config", env::temp_dir()));

    ensure_dir(&config_dir, "dwall");

    config_dir.join("dwall")
});

/// Global cache directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
/// Uses the application bundle identifier for better organization.
pub static DWALL_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let cache_dir = dirs::cache_dir().unwrap_or_else(|| temp_base("cache", env::temp_dir()));

    // bundle identifier
    ensure_dir(&cache_dir, "com.thep0y.dwall");

    cache_dir.join("com.thep0y.dwall")
});

/// Global log directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
/// Uses the application bundle identifier for better organization.
pub static DWALL_LOG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    ensure_dir(&DWALL_CACHE_DIR, "log");

    DWALL_CACHE_DIR.join("log")
});

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("dwall_lazy_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn lazy_ensure_dir_creates_and_is_idempotent() {
        let base = scratch("ensure_dir");

        ensure_dir(&base, "created");
        let created = base.join("created");
        assert!(created.is_dir(), "the directory has to be created");

        ensure_dir(&base, "created");
        assert!(
            created.is_dir(),
            "a second call must leave the directory in place"
        );

        // A file where the directory should be is logged and left alone, never a panic:
        // this runs in a GUI-subsystem process where a panic is invisible.
        let file = base.join("afile");
        fs::write(&file, b"x").unwrap();
        ensure_dir(&base, "afile");
        assert!(
            file.is_file(),
            "the existing file must survive the fallback path"
        );

        let _ = fs::remove_dir_all(&base);
    }
}
