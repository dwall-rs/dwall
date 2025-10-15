//! Filesystem infrastructure module
//!
//! This module contains low-level filesystem operations.

use std::path::{Path, PathBuf};

use dwall::{config::Config, write_config_file};
use tokio::fs;

use crate::error::DwallSettingsResult;

async fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    create_dir_if_missing(&dest).await?;

    let mut dir = fs::read_dir(src).await?;

    while let Some(entry) = dir.next_entry().await? {
        let src_path = entry.path();
        let file_type = entry.file_type().await?;
        let dest_path = dest.join(src_path.strip_prefix(src).unwrap());

        if file_type.is_dir() {
            Box::pin(copy_dir(&src_path, &dest_path)).await?;
        } else {
            fs::copy(&src_path, &dest_path).await?;
        }
    }

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

/// Moves the themes directory to a new location
pub async fn move_themes_directory(config: Config, dir_path: PathBuf) -> DwallSettingsResult<()> {
    // Validate input parameters
    if dir_path.to_path_buf().as_path() == config.themes_directory() {
        return Ok(());
    }

    // Comprehensive directory existence and permission checks
    validate_directory_move(&config, &dir_path)?;

    // Prepare new configuration
    match prepare_new_config(&config, &dir_path).await {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };

    // Perform the directory move operation
    match perform_themes_directory_move(&config, &dir_path).await {
        Ok(_) => Ok(()),
        Err(e) => {
            // Rollback configuration if move fails
            if let Err(rollback_err) = write_config_file(&config).await {
                error!(original_error = ?e, rollback_error = ?rollback_err, "Critical: Failed to rollback configuration after move failure");
            }
            e.into()
        }
    }
}

/// Validates the directory move operation
fn validate_directory_move(config: &Config, destination: &Path) -> DwallSettingsResult<()> {
    // Check if destination directory already exists
    if destination.exists() {
        return ThemeDirectoryMoveError::DestinationExists.into();
    }

    // Optional: Additional checks like write permissions could be added here
    let source = config.themes_directory();

    if !source.exists() {
        return Ok(());
    }

    if !source.is_dir() {
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
    let new_config = config.with_themes_directory(new_path);
    write_config_file(&new_config).await?;

    Ok(())
}

/// Performs the actual themes directory move
async fn perform_themes_directory_move(
    config: &Config,
    destination: &Path,
) -> Result<(), ThemeDirectoryMoveError> {
    let source = config.themes_directory();

    if !source.exists() {
        return Ok(());
    }

    copy_dir(source, destination)
        .await
        .map_err(ThemeDirectoryMoveError::CopyFailed)?;

    fs::remove_dir_all(&source)
        .await
        .map_err(ThemeDirectoryMoveError::RemoveFailed)?;

    Ok(())
}

pub async fn create_dir_if_missing<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();

    if !path.exists() {
        fs::create_dir_all(path).await
    } else {
        Ok(())
    }
}
