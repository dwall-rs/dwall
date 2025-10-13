use std::path::{Path, PathBuf};

use dwall::{config::Config, write_config_file};
use tokio::fs;

use crate::error::DwallSettingsResult;

async fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    create_dir_if_missing(&dest).await?;

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
    RemoveFailed(std::io::Error),
}

impl From<ThemeDirectoryMoveError> for DwallSettingsResult<()> {
    fn from(err: ThemeDirectoryMoveError) -> Self {
        match err {
            ThemeDirectoryMoveError::DestinationExists => {
                Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists).into())
            }
            ThemeDirectoryMoveError::CopyFailed(io_err) => Err(io_err.into()),
            ThemeDirectoryMoveError::RemoveFailed(io_err) => Err(io_err.into()),
        }
    }
}

#[tauri::command]
pub async fn move_themes_directory(config: Config, dir_path: PathBuf) -> DwallSettingsResult<()> {
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
            error!(error = %e, "Failed to prepare new configuration");
            return Err(e);
        }
    };

    // Perform the directory move operation
    match perform_themes_directory_move(&config, &dir_path).await {
        Ok(_) => {
            info!(
                from = %config.themes_directory().display(),
                to = %dir_path.display(),
                "Themes directory successfully relocated",
            );
            Ok(())
        }
        Err(e) => {
            // Rollback configuration if move fails
            warn!("Directory move failed. Attempting to rollback configuration.");
            if let Err(rollback_err) = write_config_file(&config).await {
                error!(original_error = ?e, rollback_error = ?rollback_err, "Critical: Failed to rollback configuration after move failure");
            }
            e.into()
        }
    }
}

/// Validates the directory move operation
fn validate_directory_move(config: &Config, destination: &Path) -> DwallSettingsResult<()> {
    debug!("Performing pre-move directory validation");

    // Check if destination directory already exists
    if destination.exists() {
        error!(destination = %destination.display(), "Move operation aborted: Destination directory already exists");
        return ThemeDirectoryMoveError::DestinationExists.into();
    }

    // Optional: Additional checks like write permissions could be added here
    let source = config.themes_directory();

    if !source.exists() {
        warn!(source = %source.display(), "Source directory does not exist");
        return Ok(());
    }

    if !source.is_dir() {
        error!(source = %source.display(), "Source is not a directory");
        return ThemeDirectoryMoveError::CopyFailed(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        ))
        .into();
    }

    Ok(())
}

/// Prepares the new configuration
async fn prepare_new_config(config: &Config, new_path: &Path) -> DwallSettingsResult<()> {
    debug!("Preparing new configuration with updated themes directory");

    let new_config = config.with_themes_directory(new_path);
    debug!(new_config = ?new_config, "New configuration prepared");

    write_config_file(&new_config).await?;

    Ok(())
}

/// Performs the actual themes directory move
async fn perform_themes_directory_move(
    config: &Config,
    destination: &Path,
) -> Result<(), ThemeDirectoryMoveError> {
    let source = config.themes_directory();

    debug!(source = %source.display(), destination = %destination.display(), "Initiating themes directory move");

    if !source.exists() {
        info!(source = %source.display(), "Old themes directory does not exist, skipping copy");
        return Ok(());
    }

    copy_dir(source, destination)
        .await
        .map_err(ThemeDirectoryMoveError::CopyFailed)?;

    fs::remove_dir_all(&source).await.map_err(|e| {
        error!(source = %source.display(), error = %e, "Failed to remove old themes directory");
        ThemeDirectoryMoveError::RemoveFailed(e)
    })?;

    info!(source = %source.display(), "Successfully removed old themes directory");
    Ok(())
}

pub async fn create_dir_if_missing<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    trace!(path = %path.display(), "Checking if directory exists");

    if !path.exists() {
        trace!(path = %path.display(), "Directory does not exist. Attempting to create");
        fs::create_dir_all(path)
            .await
            .map(|_| {
                info!(path = %path.display(), "Successfully created directory");
            })
            .map_err(|e| {
                error!(path = %path.display(), error = %e, "Failed to create directory");
                e
            })
    } else {
        debug!(path = %path.display(), "Directory already exists");
        Ok(())
    }
}
