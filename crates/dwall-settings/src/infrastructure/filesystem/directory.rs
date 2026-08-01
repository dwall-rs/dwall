//! Directory move operations

use std::path::{Path, PathBuf};

use tokio::fs;

use crate::error::DwallSettingsResult;

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

pub(super) async fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    super::ops::create_dir_if_missing(dest).await?;

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

/// Validates the directory move operation
fn validate_directory_move(
    source: &Path,
    destination: &Path,
) -> DwallSettingsResult<(), DirectoryMoveError> {
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

/// Performs the actual directory move
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

    fs::remove_dir_all(source)
        .await
        .map_err(DirectoryMoveError::RemoveFailed)?;

    Ok(())
}

/// Moves the directory to a new location
pub async fn move_directory(
    source: PathBuf,
    destination: PathBuf,
) -> DwallSettingsResult<(), DirectoryMoveError> {
    validate_directory_move(&source, &destination)?;
    perform_directory_move(&source, &destination).await
}
