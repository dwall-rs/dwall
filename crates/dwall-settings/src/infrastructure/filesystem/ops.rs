//! Generic filesystem operations

use std::path::{Path, PathBuf};

use tokio::fs;

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

/// Find all file paths with the specified extension in the given directory
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
