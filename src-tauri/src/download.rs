use std::{io::Cursor, path::PathBuf};

use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use crate::{config::Config, error::DwallResult, lazy::APP_CONFIG_DIR};

async fn download_theme(config: &Config, id: &str) -> DwallResult<PathBuf> {
    let github_url = format!(
        "https://github.com/thep0y/dwall-assets/releases/download/v0.1.0/{}.zip",
        id.replace(" ", ".")
    );
    let asset_url = config.github_asset_url(&github_url);

    let mut stream = reqwest::get(asset_url).await?.bytes_stream();

    let themes_dir = APP_CONFIG_DIR.join("themes");
    if !themes_dir.exists() {
        tokio::fs::create_dir(&themes_dir).await?;
    }

    let theme_zip_file = themes_dir.join(format!("{}.zip", id));
    let mut file = tokio::fs::File::open(&theme_zip_file).await?;
    while let Some(item) = stream.next().await {
        file.write(&item?);
    }

    Ok(theme_zip_file)
}

pub async fn download_theme_and_extract(config: &Config, id: &str) -> DwallResult<()> {
    let file_path = download_theme(config, id).await?;
    let archive = tokio::fs::read(file_path).await?;

    let target_dir = APP_CONFIG_DIR.join("themes").join(id);
    zip_extract::extract(Cursor::new(archive), &target_dir, true)?;

    Ok(())
}
