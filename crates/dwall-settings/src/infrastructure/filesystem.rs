//! Filesystem infrastructure module
//!
//! This module contains low-level filesystem operations.

use std::path::{Path, PathBuf};

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

#[derive(Debug, thiserror::Error)]
pub enum DirectoryMoveError {
    #[error("destination directory already exists")]
    DestinationExists,
    #[error("copy failed: {0}")]
    CopyFailed(std::io::Error),
    #[error("remove failed: {0}")]
    RemoveFailed(std::io::Error),
}

impl From<DirectoryMoveError> for DwallSettingsResult<()> {
    fn from(err: DirectoryMoveError) -> Self {
        match err {
            DirectoryMoveError::DestinationExists => {
                Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists).into())
            }
            DirectoryMoveError::CopyFailed(io_err) => Err(io_err.into()),
            DirectoryMoveError::RemoveFailed(io_err) => Err(io_err.into()),
        }
    }
}

/// Moves the directory to a new location
pub async fn move_directory(
    source: PathBuf,
    destination: PathBuf,
) -> DwallSettingsResult<(), DirectoryMoveError> {
    // Comprehensive directory existence and permission checks
    validate_directory_move(&source, &destination)?;

    // Perform the directory move operation
    perform_directory_move(&source, &destination).await
}

/// Validates the directory move operation
fn validate_directory_move(
    source: &Path,
    destination: &Path,
) -> DwallSettingsResult<(), DirectoryMoveError> {
    // Check if destination directory already exists
    if destination.exists() {
        return Err(DirectoryMoveError::DestinationExists);
    }

    if !source.exists() {
        return Ok(());
    }

    if !source.is_dir() {
        return Err(DirectoryMoveError::CopyFailed(std::io::Error::new(
            std::io::ErrorKind::NotADirectory,
            "Source is not a directory",
        )));
    }

    Ok(())
}

/// Performs the actual themes directory move
async fn perform_directory_move(
    source: &Path,
    destination: &Path,
) -> Result<(), DirectoryMoveError> {
    if !source.exists() {
        return Ok(());
    }

    copy_dir(source, destination)
        .await
        .map_err(DirectoryMoveError::CopyFailed)?;

    fs::remove_dir_all(&source)
        .await
        .map_err(DirectoryMoveError::RemoveFailed)?;

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

/// List all immediate subdirectories in the specified directory
pub async fn list_subdirectories(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut subdirs = Vec::new();
    let mut dir = fs::read_dir(path).await?;

    while let Some(entry) = dir.next_entry().await? {
        let file_type = entry.file_type().await?;
        if file_type.is_dir() {
            subdirs.push(entry.path());
        }
    }

    Ok(subdirs)
}

/// Find all file paths with the specified extension in the given directory
///
/// Does not recurse into subdirectories.
pub async fn find_files_in_dir(dir: &Path, extension: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut dir = fs::read_dir(dir).await?;

    while let Some(entry) = dir.next_entry().await? {
        // Only consider regular files (or symlinks to files), ignore directories
        if entry.file_type().await?.is_file() {
            let path = entry.path();
            // Check extension
            if let Some(ext) = path.extension()
                && ext.eq_ignore_ascii_case(extension)
            {
                files.push(path);
            }
        }
    }

    Ok(files)
}
