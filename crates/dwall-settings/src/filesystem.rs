//! Filesystem helpers: directory creation, listing and moves.

use std::path::{Component, Path, PathBuf};

use tokio::fs;

use crate::error::{DwallSettingsError, DwallSettingsResult};

// ── Path validation ─────────────────────────────────────────────────────────

/// Validates a directory path that came from the frontend.
///
/// The value is used for recursive filesystem operations, so it must be an
/// absolute path without `..` components. (The user picks it with a native folder
/// dialog, so the allowed root cannot be narrowed further here.)
pub fn validate_directory(path: &Path, label: &str) -> DwallSettingsResult<()> {
    if !path.is_absolute() {
        return Err(DwallSettingsError::Other(format!(
            "{label} must be an absolute path, got '{}'",
            path.display()
        )));
    }

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(DwallSettingsError::Other(format!(
            "{label} must not contain '..': '{}'",
            path.display()
        )));
    }

    Ok(())
}

/// Validates a theme identifier before it is joined onto the themes directory.
///
/// Theme ids are directory names: anything containing a separator, a drive/stream
/// colon, a traversal component or a control character could escape the themes
/// directory, and the downloader runs `remove_dir_all` on the resulting path.
pub fn validate_theme_id(theme_id: &str) -> DwallSettingsResult<()> {
    let invalid = theme_id.is_empty()
        || theme_id.len() > 128
        || theme_id == "."
        || theme_id == ".."
        || theme_id.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        });

    if invalid {
        return Err(DwallSettingsError::Other(format!(
            "Invalid theme id: '{theme_id}'"
        )));
    }

    Ok(())
}

// ── Generic operations ──────────────────────────────────────────────────────

/// Creates directories if they don't exist
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

/// Find all file paths with the specified extension in the given directory.
///
/// Does not recurse into subdirectories.
pub async fn find_files_in_dir(dir: &Path, extension: &str) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut dir = fs::read_dir(dir).await?;

    while let Some(entry) = dir.next_entry().await? {
        if entry.file_type().await?.is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension()
                && ext.eq_ignore_ascii_case(extension)
            {
                files.push(path);
            }
        }
    }

    Ok(files)
}

// ── Directory move ──────────────────────────────────────────────────────────

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

async fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    create_dir_if_missing(dest).await?;

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

fn validate_directory_move(source: &Path, destination: &Path) -> Result<(), DirectoryMoveError> {
    if destination.exists() {
        return Err(DirectoryMoveError::DestinationExists);
    }

    // Moving a directory into itself would copy forever and then delete the copy.
    if destination.starts_with(source) {
        return Err(DirectoryMoveError::CopyFailed(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Destination is inside the source directory",
        )));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_theme_id_accepts_real_theme_names() {
        for id in ["Big Sur", "Big.Sur", "Solar-Gradients_2", "地球"] {
            assert!(validate_theme_id(id).is_ok(), "{id} should be accepted");
        }
    }

    #[test]
    fn validate_theme_id_rejects_traversal_and_separators() {
        // The id becomes a directory name and the downloader clears that directory.
        for id in [
            "",
            ".",
            "..",
            "../evil",
            "..\\evil",
            "a/b",
            "a\\b",
            "C:\\Windows",
            "C:evil",
            "a:b",
            "a*b",
            "a?b",
            "a\"b",
            "a<b",
            "a>b",
            "a|b",
            "a\nb",
        ] {
            assert!(validate_theme_id(id).is_err(), "{id:?} should be rejected");
        }
    }

    #[test]
    fn validate_theme_id_rejects_overlong_names() {
        assert!(validate_theme_id(&"a".repeat(128)).is_ok());
        assert!(validate_theme_id(&"a".repeat(129)).is_err());
    }

    #[test]
    fn validate_directory_requires_absolute_paths_without_parent_dir() {
        let absolute = std::env::temp_dir();

        assert!(validate_directory(&absolute, "themes").is_ok());
        assert!(validate_directory(Path::new("themes"), "themes").is_err());
        assert!(validate_directory(&absolute.join("..").join("other"), "themes").is_err());
        assert!(validate_directory(&absolute.join("nested").join("deep"), "themes").is_ok());
    }

    #[test]
    fn validate_directory_move_refuses_destination_inside_source() {
        let source = std::env::temp_dir().join("dwall_move_source");

        assert!(matches!(
            validate_directory_move(&source, &source.join("inner")),
            Err(DirectoryMoveError::CopyFailed(_))
        ));
    }
}
