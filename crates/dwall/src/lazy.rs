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
            error!("Failed to create config directory: {e}");
            panic!("Failed to create config directory: {e}");
        } else {
            info!(
                "Config directory created successfully: {}",
                app_config_dir.display()
            );
        }
    } else {
        debug!(
            "Config directory already exists: {}",
            app_config_dir.display()
        );
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
    trace!("Initializing cache directory: {}", dir.display());

    if !dir.exists() {
        if let Err(e) = fs::create_dir(&dir) {
            error!("Failed to create cache directory: {e}");
            panic!("Failed to create cache directory: {e}");
        } else {
            info!("Cache directory created successfully at: {}", dir.display());
        }
    } else {
        debug!("Cache directory already exists: {}", dir.display());
    }

    dir
});

/// Global log directory path
///
/// Initialized lazily on first access. Creates the directory if it doesn't exist.
/// Uses the application bundle identifier for better organization.
///
/// # Panics
/// Panics if unable to determine user's log directory or create the dwall subdirectory.
pub static DWALL_LOG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let log_dir = DWALL_CACHE_DIR.join("log");

    if !log_dir.exists() {
        if let Err(e) = fs::create_dir(&log_dir) {
            error!("Failed to create log directory: {e}");
            panic!("Failed to create log directory: {e}");
        } else {
            info!(
                "Log directory created successfully at: {}",
                log_dir.display()
            );
        }
    } else {
        debug!("Log directory already exists: {}", log_dir.display());
    }

    log_dir
});
