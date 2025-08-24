use std::{fs, path::PathBuf, sync::LazyLock};

pub static DWALL_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::config_dir().unwrap();

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

pub static DWALL_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::cache_dir().unwrap();

    let dir = config_dir.join("com.thep0y.dwall"); // bundle identifier
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
