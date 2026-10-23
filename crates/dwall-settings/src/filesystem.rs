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
/// directory, and the downloader runs `remove_dir_all` on the resulting path. A name
/// Windows will not keep as written — a reserved device name, a trailing dot or space —
/// is refused too, because the directory that lands on disk would not be the one the id
/// names.
pub fn validate_theme_id(theme_id: &str) -> DwallSettingsResult<()> {
    let invalid = theme_id.is_empty()
        || theme_id.len() > 128
        || theme_id == "."
        || theme_id == ".."
        || is_unwritable_name(theme_id)
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

// ── Windows name rules ──────────────────────────────────────────────────────

/// Whether Windows refuses to write `name` as an ordinary file in a directory.
///
/// A colon opens an alternate data stream instead of a file, `CON`/`NUL`/`COM1`
/// address a device rather than a path, and a trailing dot or space is silently
/// dropped. Anything named that way ends up somewhere other than where it says, which
/// is why both an archive entry and a theme id are checked against this before a path
/// is built from them.
pub(crate) fn is_unwritable_name(name: &str) -> bool {
    if name.is_empty() || name.ends_with('.') || name.ends_with(' ') || name.contains(':') {
        return true;
    }

    // The device names stay reserved with any extension (`NUL.txt` is still the NUL
    // device) but only up to nine, and only as the whole stem: `COM10` and `CONSOLE`
    // are ordinary files.
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();

    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

/// Validates the two theme roots a configuration carries.
///
/// Both roots are used for recursive filesystem operations: the downloader clears
/// and extracts under `themes_directory`, and the migration command copies a tree
/// and removes the source. Every command that acts on a renderer-supplied
/// configuration therefore has to validate them the same way — keeping the two
/// calls in one place is what stops a new command (or an old one that forgot) from
/// being the single path that writes under an unchecked root.
pub fn validate_theme_roots(config: &dwall::Config) -> DwallSettingsResult<()> {
    validate_directory(config.themes_directory(), "themes_directory")?;
    validate_directory(
        config.customized_themes_directory(),
        "customized_themes_directory",
    )?;

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
    if is_inside(destination, source) {
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

/// Whether `destination` is `source` or lives inside it.
///
/// `Path::starts_with` compares components without regard to case, and Windows
/// paths are case-insensitive: moving `D:\Themes` to `d:\themes\new` passed the
/// check, so `copy_dir` recursed into the directory it had just created inside the
/// source and the final `remove_dir_all(source)` deleted the destination with it.
/// The comparison is therefore case-folded, and against the resolved forms as well
/// so a junction or an 8.3 name cannot hide the same relationship.
fn is_inside(destination: &Path, source: &Path) -> bool {
    if comparable_components(destination).starts_with(&comparable_components(source)) {
        return true;
    }

    match (resolved_form(source), resolved_form(destination)) {
        (Some(source), Some(destination)) => {
            comparable_components(&destination).starts_with(&comparable_components(&source))
        }
        _ => false,
    }
}

/// The path's components, case-folded, for comparison.
fn comparable_components(path: &Path) -> Vec<String> {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
        .collect()
}

/// The path with its longest existing prefix resolved, or `None` when even that
/// fails (a missing drive, an unreadable parent).
///
/// The destination usually does not exist yet, so it cannot be canonicalised
/// directly; resolving the part that does and appending the rest is what makes the
/// comparison see through junctions.
fn resolved_form(path: &Path) -> Option<PathBuf> {
    let mut existing = path;
    let mut remaining: Vec<&std::ffi::OsStr> = Vec::new();

    loop {
        if let Ok(canonical) = existing.canonicalize() {
            let mut resolved = canonical;
            for component in remaining.iter().rev() {
                resolved.push(component);
            }

            return Some(resolved);
        }

        remaining.push(existing.file_name()?);
        existing = existing.parent()?;
    }
}

/// The sibling directory a move is staged in, so the destination only ever appears
/// complete.
///
/// The name is derived from the destination rather than randomised: a crash between
/// the copy and the rename leaves it behind, and the next attempt has to find and
/// clear exactly the same directory.
fn staging_directory(destination: &Path) -> PathBuf {
    let mut name = destination.file_name().unwrap_or_default().to_os_string();
    name.push(".dwall-partial");
    destination.with_file_name(name)
}

async fn perform_directory_move(
    source: &Path,
    destination: &Path,
) -> Result<(), DirectoryMoveError> {
    if !source.exists() {
        return Ok(());
    }

    let staging = staging_directory(destination);

    // A previous attempt that died between the copy and the rename left this behind;
    // it is ours by construction, so clearing it is not destructive.
    if staging.exists() {
        let _ = fs::remove_dir_all(&staging).await;
    }

    // Copy beside the destination and rename it into place: a copy that fails halfway
    // leaves the destination untouched, so the move can be retried instead of failing
    // on a half-written tree that `DestinationExists` then refuses forever.
    if let Err(err) = copy_dir(source, &staging).await {
        let _ = fs::remove_dir_all(&staging).await;
        return Err(DirectoryMoveError::CopyFailed(err));
    }

    if let Err(err) = fs::rename(&staging, destination).await {
        let _ = fs::remove_dir_all(&staging).await;
        return Err(DirectoryMoveError::CopyFailed(err));
    }

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
    fn validate_theme_id_rejects_names_windows_would_not_keep() {
        for theme_id in ["CON", "nul.txt", "Trailing.", "Trailing "] {
            assert!(
                validate_theme_id(theme_id).is_err(),
                "'{theme_id}' is not a name Windows keeps as written"
            );
        }
    }

    #[test]
    fn validate_theme_id_accepts_names_that_only_look_reserved() {
        for theme_id in ["CONSOLE", "COM10", "Sun Rise", "昼与夜"] {
            assert!(
                validate_theme_id(theme_id).is_ok(),
                "'{theme_id}' is usable"
            );
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

    // ── validate_theme_roots ────────────────────────────────────────────────

    /// A configuration whose two theme roots are exactly what the renderer sent.
    ///
    /// Built through `serde` because the fields are private: this is the same path
    /// a Tauri command argument takes.
    fn config_with_roots(themes: &str, customized: &str) -> dwall::Config {
        let json = format!(
            r#"{{"themes_directory":{},"customized_themes_directory":{}}}"#,
            serde_json::to_string(themes).unwrap(),
            serde_json::to_string(customized).unwrap(),
        );

        serde_json::from_str(&json).unwrap()
    }

    fn absolute_pair() -> (String, String) {
        let root = std::env::temp_dir();

        (
            root.join("themes").to_string_lossy().into_owned(),
            root.join("customize").to_string_lossy().into_owned(),
        )
    }

    #[test]
    fn validate_theme_roots_accepts_absolute_directories() {
        let (themes, customized) = absolute_pair();

        assert!(validate_theme_roots(&config_with_roots(&themes, &customized)).is_ok());
    }

    /// `download_theme_cmd` used to skip this check and go straight to
    /// `remove_dir_all` + extraction under whatever root the renderer sent.
    #[test]
    fn validate_theme_roots_rejects_a_relative_themes_directory() {
        let (_, customized) = absolute_pair();

        assert!(validate_theme_roots(&config_with_roots("themes", &customized)).is_err());
    }

    #[test]
    fn validate_theme_roots_rejects_a_relative_customized_directory() {
        let (themes, _) = absolute_pair();

        assert!(validate_theme_roots(&config_with_roots(&themes, "customize")).is_err());
    }

    #[test]
    fn validate_theme_roots_rejects_a_parent_dir_component() {
        let (themes, _) = absolute_pair();
        let escaping = std::env::temp_dir()
            .join("..")
            .join("elsewhere")
            .to_string_lossy()
            .into_owned();

        assert!(validate_theme_roots(&config_with_roots(&themes, &escaping)).is_err());
        assert!(validate_theme_roots(&config_with_roots(&escaping, &themes)).is_err());
    }

    // ── is_inside ───────────────────────────────────────────────────────────

    #[test]
    fn a_destination_below_the_source_is_inside_it() {
        assert!(is_inside(
            Path::new(r"D:\Themes\new"),
            Path::new(r"D:\Themes")
        ));
        assert!(is_inside(Path::new(r"D:\Themes"), Path::new(r"D:\Themes")));
    }

    /// The case that used to be missed, and cost the destination: the copy
    /// recursed into a directory it had just created inside the source, and the
    /// removal at the end deleted both.
    #[test]
    fn the_comparison_ignores_case_like_windows_does() {
        assert!(is_inside(
            Path::new(r"d:\themes\new"),
            Path::new(r"D:\Themes")
        ));
        assert!(is_inside(
            Path::new(r"D:\THEMES\New"),
            Path::new(r"d:\themes")
        ));
    }

    #[test]
    fn a_sibling_with_a_shared_prefix_is_not_inside() {
        // Component-wise, so `Themes2` is not inside `Themes`.
        assert!(!is_inside(
            Path::new(r"D:\Themes2"),
            Path::new(r"D:\Themes")
        ));
        assert!(!is_inside(
            Path::new(r"D:\Other\Themes"),
            Path::new(r"D:\Themes")
        ));
        assert!(!is_inside(Path::new(r"E:\Themes"), Path::new(r"D:\Themes")));
    }

    #[test]
    fn a_real_destination_below_a_real_source_is_refused() {
        // The end-to-end shape of the bug: both paths exist on disk, with the
        // destination nested in the source.
        let source = std::env::temp_dir().join("dwall_move_guard");
        let destination = source.join("inner");
        let _ = std::fs::remove_dir_all(&source);
        std::fs::create_dir_all(&source).unwrap();

        assert!(matches!(
            validate_directory_move(&source, &destination),
            Err(DirectoryMoveError::CopyFailed(_))
        ));

        let _ = std::fs::remove_dir_all(&source);
    }

    /// The same guard, through the entry point the command uses, with the case
    /// difference that used to slip past it.
    #[test]
    fn validate_directory_move_refuses_a_case_varied_destination() {
        assert!(matches!(
            validate_directory_move(Path::new(r"D:\Themes"), Path::new(r"d:\themes\new")),
            Err(DirectoryMoveError::CopyFailed(_))
        ));
    }

    // ── move_directory ──────────────────────────────────────────────────────

    /// A source tree with a top-level file and a nested one, inside a parent that
    /// also holds the destination.
    fn scratch_tree(name: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(name);
        let source = root.join("source");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(source.join("nested")).unwrap();
        std::fs::write(source.join("a.txt"), b"a").unwrap();
        std::fs::write(source.join("nested").join("b.txt"), b"b").unwrap();

        (root, source)
    }

    #[tokio::test]
    async fn move_directory_moves_a_tree_and_removes_the_source() {
        let (root, source) = scratch_tree("dwall_move_ok");
        let destination = root.join("destination");

        move_directory(source.clone(), destination.clone())
            .await
            .unwrap();

        assert!(!source.exists(), "the source should be gone");
        assert_eq!(std::fs::read(destination.join("a.txt")).unwrap(), b"a");
        assert_eq!(
            std::fs::read(destination.join("nested").join("b.txt")).unwrap(),
            b"b"
        );
        assert!(
            !staging_directory(&destination).exists(),
            "the staging directory must not survive a successful move"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    /// A move that died between the copy and the rename used to leave the destination
    /// half written, and the next attempt refused it with `DestinationExists`.
    #[tokio::test]
    async fn a_leftover_staging_directory_does_not_block_the_next_move() {
        let (root, source) = scratch_tree("dwall_move_stale");
        let destination = root.join("destination");
        let staging = staging_directory(&destination);

        std::fs::create_dir_all(&staging).unwrap();
        std::fs::write(staging.join("half-written.txt"), b"junk").unwrap();

        move_directory(source.clone(), destination.clone())
            .await
            .unwrap();

        assert_eq!(std::fs::read(destination.join("a.txt")).unwrap(), b"a");
        assert!(
            !destination.join("half-written.txt").exists(),
            "the stale staging contents must not reach the destination"
        );
        assert!(!staging.exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn move_directory_refuses_an_existing_destination_and_keeps_the_source() {
        let (root, source) = scratch_tree("dwall_move_taken");
        let destination = root.join("destination");
        std::fs::create_dir_all(&destination).unwrap();

        assert!(matches!(
            move_directory(source.clone(), destination.clone()).await,
            Err(DirectoryMoveError::DestinationExists)
        ));
        assert!(source.exists(), "the source must survive a refused move");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn move_directory_with_a_missing_source_is_a_no_op() {
        let root = std::env::temp_dir().join("dwall_move_missing");
        let _ = std::fs::remove_dir_all(&root);

        move_directory(root.join("absent"), root.join("destination"))
            .await
            .unwrap();

        assert!(!root.join("destination").exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
