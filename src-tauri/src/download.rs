use std::{io::Cursor, path::PathBuf};

use futures_util::StreamExt;
use tokio::{fs, io::AsyncWriteExt};

use crate::{config::Config, error::DwallResult, lazy::APP_CONFIG_DIR, theme::THEMES_DIR};

async fn download_theme<'a>(config: &Config<'a>, id: &str) -> DwallResult<PathBuf> {
    let github_url = format!(
        "https://github.com/thep0y/dwall-assets/releases/download/v0.1.0/{}.zip",
        id.replace(" ", ".")
    );
    let asset_url = config.github_asset_url(&github_url);

    let mut stream = reqwest::get(asset_url).await?.bytes_stream();

    fs::remove_dir_all(&*THEMES_DIR).await?;
    fs::create_dir(&*THEMES_DIR).await?;

    let theme_zip_file = THEMES_DIR.join(format!("{}.zip", id));
    let mut file = fs::File::open(&theme_zip_file).await?;
    while let Some(item) = stream.next().await {
        file.write_all(&item?).await?;
    }

    Ok(theme_zip_file)
}

pub async fn download_theme_and_extract<'a>(config: &Config<'a>, id: &str) -> DwallResult<()> {
    let file_path = download_theme(config, id).await?;
    let archive = fs::read(file_path).await?;

    let target_dir = APP_CONFIG_DIR.join("themes").join(id);
    zip_extract::extract(Cursor::new(archive), &target_dir, true)?;

    Ok(())
}
