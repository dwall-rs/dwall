use std::{fs, path::PathBuf, sync::LazyLock};

/// Global configuration directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
///
/// # Panics
/// Panics if unable to determine user's config directory or create the dwall subdirectory.
/// Consider using a Result-based approach for better error handling in production.
pub static DWALL_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::config_dir().expect("Failed to determine user config directory");

    let app_config_dir = config_dir.join("dwall");

    if !app_config_dir.exists() {
        if let Err(e) = fs::create_dir(&app_config_dir) {
            error!(error = %e, "Failed to create config directory");
            panic!("Failed to create config directory: {e}");
        } else {
            info!(path = %app_config_dir.display(), "Config directory created successfully");
        }
    } else {
        debug!(path = %app_config_dir.display(), "Config directory already exists");
    }

    app_config_dir
});

/// Global cache directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
/// Uses the application bundle identifier for better organization.
///
/// # Panics
/// Panics if unable to determine user's cache directory or create the dwall subdirectory.
pub static DWALL_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let cache_dir = dirs::cache_dir().expect("Failed to determine user cache directory");

    let dir = cache_dir.join("com.thep0y.dwall"); // bundle identifier
    trace!(path = %dir.display(), "Initializing cache directory");

    if !dir.exists() {
        if let Err(e) = fs::create_dir(&dir) {
            error!(error = %e, "Failed to create cache directory");
            panic!("Failed to create cache directory: {e}");
        } else {
            info!("Cache directory created successfully at: {}", dir.display());
        }
    } else {
        debug!(path = %dir.display(), "Cache directory already exists");
    }

    dir
});
