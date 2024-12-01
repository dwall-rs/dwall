use std::{fs, path::PathBuf, sync::LazyLock};

pub static APP_CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    let config_dir = dirs::config_dir().unwrap();

    let app_config_dir = config_dir.join("dwall");

    if !app_config_dir.exists() {
        fs::create_dir(&app_config_dir).unwrap();
    }

    app_config_dir
});
