use std::{fs, path::PathBuf, sync::LazyLock};

pub static DWALL_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::config_dir().unwrap();

    let app_config_dir = config_dir.join("dwall");

    if !app_config_dir.exists() {
        fs::create_dir(&app_config_dir).unwrap();
    }

    app_config_dir
});

pub static DWALL_CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::cache_dir().unwrap();

    let dir = config_dir.join("dwall");
    trace!("Initializing cache directory at: {}", dir.display());

    if !dir.exists() {
        if let Err(e) = fs::create_dir(&dir) {
            let error_message = format!("Failed to create cache dir: {}", e);
            error!("{}", error_message);
            panic!("{}", error_message);
        } else {
            info!("Cache directory created successfully at: {}", dir.display());
        }
    } else {
        debug!("Cache directory already exists at: {}", dir.display());
    }

    dir
});
