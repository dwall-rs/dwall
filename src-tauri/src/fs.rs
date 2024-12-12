use std::borrow::Cow;
use std::path::Path;

use dwall::config::{write_config_file, Config};
use tokio::fs;
use tokio::io;

use crate::error::DwallSettingsResult;

async fn copy_dir(src: &Path, dest: &Path) -> io::Result<()> {
    fs::create_dir_all(dest).await.map_err(|e| {
        error!(path = %dest.display(), error = %e, "Failed to create directory");
        e
    })?;

    let mut dir = fs::read_dir(src).await.map_err(|e| {
        error!(path = %src.display(), error = %e, "Failed to read source directory");
        e
    })?;

    while let Some(entry) = dir.next_entry().await.map_err(|e| {
        error!(path = %src.display(), error = %e, "Failed to get next entry in source directory");
        e
    })? {
        let src_path = entry.path();
        let file_type = entry.file_type().await.map_err(|e| {
            error!(path = %src_path.display(), error = %e, "Failed to get file type of entry");
            e
        })?;
        let dest_path = dest.join(src_path.strip_prefix(src).unwrap());

        if file_type.is_dir() {
            info!(src = %src_path.display(), dest = %dest_path.display(), "Recursively copying directory");
            Box::pin(copy_dir(&src_path, &dest_path)).await?;
        } else {
            info!(src = %src_path.display(), dest = %dest_path.display(), "Copying file");
            fs::copy(&src_path, &dest_path).await.map_err(|e| {
                error!(src = %src_path.display(), dest = %dest_path.display(), error = %e, "Failed to copy file");
                e
            })?;
        }
    }

    info!(src = %src.display(), dest = %dest.display(), "Directory copy completed successfully");

    Ok(())
}

#[derive(Debug)]
enum ThemeDirectoryMoveError {
    DestinationExists,
    CopyFailed(std::io::Error),
}

impl From<ThemeDirectoryMoveError> for DwallSettingsResult<()> {
    fn from(err: ThemeDirectoryMoveError) -> Self {
        match err {
            ThemeDirectoryMoveError::DestinationExists => {
                Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists).into())
            }
            ThemeDirectoryMoveError::CopyFailed(io_err) => Err(io_err.into()),
        }
    }
}

#[tauri::command]
pub async fn move_themes_directory(
    config: Config<'_>,
    dir_path: Cow<'_, Path>,
) -> DwallSettingsResult<()> {
    // Validate input parameters
    if dir_path.to_path_buf().as_path() == config.themes_directory() {
        debug!("Source and destination paths are the same. No move necessary.");
        return Ok(());
    }

    // Comprehensive directory existence and permission checks
    validate_directory_move(&config, &dir_path)?;

    // Prepare new configuration
    match prepare_new_config(&config, &dir_path).await {
        Ok(_) => {}
        Err(e) => {
            error!("Failed to prepare new configuration: {:?}", e);
            return Err(e);
        }
    };

    // Perform the directory move operation
    match perform_themes_directory_move(&config, &dir_path).await {
        Ok(_) => {
            info!(
                "Themes directory successfully relocated from {} to {}",
                config.themes_directory().display(),
                dir_path.display()
            );
            Ok(())
        }
        Err(e) => {
            // Rollback configuration if move fails
            warn!("Directory move failed. Attempting to rollback configuration.");
            if let Err(rollback_err) = write_config_file(&config).await {
                error!(
                    "Critical: Failed to rollback configuration after move failure. \
                    Original error: {:?}, Rollback error: {:?}",
                    e, rollback_err
                );
            }
            e.into()
        }
    }
}

/// Validates the directory move operation
fn validate_directory_move(config: &Config<'_>, destination: &Path) -> DwallSettingsResult<()> {
    debug!("Performing pre-move directory validation");

    // Check if destination directory already exists
    if destination.exists() {
        error!(
            "Move operation aborted: Destination directory already exists at {}",
            destination.display()
        );
        return ThemeDirectoryMoveError::DestinationExists.into();
    }

    // Optional: Additional checks like write permissions could be added here
    let source = config.themes_directory();
    if !source.is_dir() {
        error!("Source is not a directory: {}", source.display());
        return ThemeDirectoryMoveError::CopyFailed(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Source is not a directory",
        ))
        .into();
    }

    Ok(())
}

/// Prepares the new configuration
async fn prepare_new_config<'a>(
    config: &Config<'a>,
    new_path: &'a Path,
) -> DwallSettingsResult<()> {
    debug!("Preparing new configuration with updated themes directory");

    let new_config = config.with_themes_directory(new_path);
    debug!("new config: {:?}", new_config);

    write_config_file(&new_config).await?;

    Ok(())
}

/// Performs the actual themes directory move
async fn perform_themes_directory_move(
    config: &Config<'_>,
    destination: &Path,
) -> Result<(), ThemeDirectoryMoveError> {
    let source = config.themes_directory();

    debug!(
        "Initiating themes directory move from {} to {}",
        source.display(),
        destination.display()
    );

    copy_dir(source, destination)
        .await
        .map_err(ThemeDirectoryMoveError::CopyFailed)?;

    match fs::remove_dir_all(source).await {
        Ok(_) => {
            info!(
                "Successfully removed old themes directory: {}",
                source.display()
            );
        }
        Err(e) => {
            error!(
                "Failed to remove old themes directory {}: {}",
                source.display(),
                e
            );
        }
    };

    Ok(())
}
