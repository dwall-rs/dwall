//! Zip archive extraction.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tokio::fs as async_fs;
use zip::ZipArchive;
use zip::read::root_dir_common_filter;

use crate::error::{DwallSettingsError, DwallSettingsResult};

/// Bytes an archive may expand to, and the most this module is willing to write.
///
/// The downloader bounds the archive it fetches by the same number: a compressed
/// archive is never larger than what it expands to, so this is the largest file that
/// could still be extracted.
pub(crate) const MAX_EXTRACTED_BYTES: u64 = 8 * 1024 * 1024 * 1024; // 8 GiB

/// Upper bound on the number of entries, so a zip bomb cannot create millions of
/// files either.
const MAX_ARCHIVE_ENTRIES: usize = 16_384;

/// Copy buffer for one entry.
const COPY_BUFFER_BYTES: usize = 64 * 1024;

/// Extract a zip archive to the target directory
///
/// Runs on a blocking thread (deflate + file writes) and streams from the archive
/// file instead of loading it into memory.
pub async fn extract_zip(zip_path: &Path, target_dir: &Path) -> DwallSettingsResult<()> {
    let zip_path = zip_path.to_path_buf();
    let target_dir = target_dir.to_path_buf();

    tauri::async_runtime::spawn_blocking(move || extract_zip_blocking(&zip_path, &target_dir))
        .await
        .map_err(|e| DwallSettingsError::Other(e.to_string()))?
}

fn extract_zip_blocking(zip_path: &Path, target_dir: &Path) -> DwallSettingsResult<()> {
    extract_with_limit(zip_path, target_dir, MAX_EXTRACTED_BYTES)
}

/// Extracts `zip_path` into `target_dir`, writing at most `max_bytes`.
///
/// Every entry goes through [`copy_bounded`], so `max_bytes` bounds what actually
/// reaches the disk. Summing the sizes the central directory *declares* and then
/// handing the archive to the crate's extractor — which copies each entry without a
/// limit — let an archive that declares 1 KiB per entry and expands a thousandfold
/// pass the check and fill the disk.
fn extract_with_limit(
    zip_path: &Path,
    target_dir: &Path,
    max_bytes: u64,
) -> DwallSettingsResult<()> {
    let file = File::open(zip_path).map_err(|e| {
        error!(
            zip_path = %zip_path.display(),
            error = %e,
            "Failed to open zip archive"
        );
        e
    })?;

    let mut zip = ZipArchive::new(file)?;

    if zip.len() > MAX_ARCHIVE_ENTRIES {
        return Err(DwallSettingsError::Other(format!(
            "Archive has too many entries ({} > {MAX_ARCHIVE_ENTRIES})",
            zip.len()
        )));
    }

    // The declared sizes first: reading them decompresses nothing and rejects an
    // honest bomb before the first byte is written. The limit below is what the
    // guarantee rests on, and the two report different messages so a test can tell
    // which one fired.
    if declared_size(&mut zip)? > max_bytes {
        return Err(DwallSettingsError::Other(format!(
            "Archive declares more than {} GiB",
            MAX_EXTRACTED_BYTES / (1024 * 1024 * 1024)
        )));
    }

    // The single top-level directory every relevant entry shares, when there is
    // one: the crate's own detection, so archives keep unpacking into the same
    // layout they did under `extract_unwrapped_root_dir`.
    let root = zip.root_dir(root_dir_common_filter)?;

    fs::create_dir_all(target_dir).map_err(|e| {
        error!(
            target_dir = %target_dir.display(),
            error = %e,
            "Failed to create the extraction directory"
        );
        e
    })?;

    let mut written = 0u64;

    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;

        let Some(path) = entry.enclosed_name() else {
            let name = entry.name()?;
            error!(
                entry = name,
                "Archive entry resolves to a path outside the extraction directory"
            );
            return Err(DwallSettingsError::Other(format!(
                "Archive entry '{}' escapes the extraction directory",
                name
            )));
        };

        let Some(relative) = relative_destination(&path, root.as_deref()) else {
            // Filtered junk (`.DS_Store`, `Thumbs.db`, `__MACOSX/**`) or the root
            // directory entry itself, which the unwrapping already accounts for.
            continue;
        };

        let out_path = target_dir.join(&relative);

        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if entry.is_symlink() {
            // Writing the link's target as a regular file would be pointless for a
            // theme, and turning an archive entry into a real symlink is exactly the
            // escape this module exists to prevent.
            warn!(
                entry = entry.name(),
                "Skipping a symlink entry in the archive"
            );
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out = File::create(&out_path).map_err(|e| {
            error!(
                path = %out_path.display(),
                error = %e,
                "Failed to create an extracted file"
            );
            e
        })?;

        written += match copy_bounded(&mut entry, &mut out, max_bytes.saturating_sub(written)) {
            Ok(copied) => copied,
            Err(e) => {
                // A refusal must not leave a truncated entry behind: the caller
                // clears the staging directory, but nothing downstream should have
                // to reason about a half-written file.
                let _ = fs::remove_file(&out_path);
                return Err(e);
            }
        };
    }

    info!(
        target_dir = %target_dir.display(),
        written_bytes = written,
        "Successfully extracted archive"
    );

    Ok(())
}

/// Sum of the uncompressed sizes the archive's central directory declares.
fn declared_size(zip: &mut ZipArchive<File>) -> DwallSettingsResult<u64> {
    let mut declared_bytes = 0u64;

    for index in 0..zip.len() {
        declared_bytes = declared_bytes.saturating_add(zip.by_index(index)?.size());
    }

    Ok(declared_bytes)
}

fn expanded_too_much_error() -> DwallSettingsError {
    DwallSettingsError::Other(format!(
        "Archive expands to more than {} GiB",
        MAX_EXTRACTED_BYTES / (1024 * 1024 * 1024)
    ))
}

/// Where an entry that resolved to `path` is written, relative to the target.
///
/// `None` means the entry is left out: with a root directory, entries the root
/// filter excluded are not under it, and the root directory entry itself strips
/// down to nothing.
fn relative_destination(path: &Path, root: Option<&Path>) -> Option<PathBuf> {
    let relative = match root {
        Some(root) => path.strip_prefix(root).ok()?.to_path_buf(),
        None => path.to_path_buf(),
    };

    if relative.as_os_str().is_empty() {
        return None;
    }

    Some(relative)
}

/// Copies `reader` into `writer`, refusing to write more than `limit` bytes.
///
/// The limit is checked before each chunk is written, so a refusal never writes
/// past it, and the returned count is what really reached the writer.
fn copy_bounded(
    reader: &mut impl Read,
    writer: &mut impl Write,
    limit: u64,
) -> DwallSettingsResult<u64> {
    let mut buffer = [0u8; COPY_BUFFER_BYTES];
    let mut written = 0u64;

    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            return Ok(written);
        }

        if written + read as u64 > limit {
            error!(
                limit = limit,
                "Archive entry expands past the remaining size limit"
            );
            return Err(expanded_too_much_error());
        }

        writer.write_all(&buffer[..read])?;
        written += read as u64;
    }
}

/// Remove a file (used for cleanup after extraction)
///
/// A file that is already gone counts as removed: the callers use this to clear an
/// intermediate archive, and the step that failed may have taken it with it.
pub async fn remove_file(path: &Path) -> DwallSettingsResult<()> {
    match async_fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => {
            error!(
                path = %path.display(),
                error = %e,
                "Failed to remove file"
            );
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use zip::CompressionMethod;
    use zip::write::SimpleFileOptions;

    use super::*;

    /// A zip archive built in memory from `(name, contents)` pairs.
    fn archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        for (name, contents) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(contents).unwrap();
        }

        writer.finish().unwrap().into_inner()
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_archive_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    /// Rewrites the uncompressed size the central directory declares, leaving the
    /// deflate payload alone: what a crafted archive does to mislead a pre-check.
    fn lie_about_uncompressed_size(bytes: &mut [u8], declared: u32) {
        // Central directory file header: the uncompressed size sits 24 bytes in.
        let signature = [0x50, 0x4b, 0x01, 0x02];
        let offset = bytes
            .windows(signature.len())
            .position(|window| window == signature)
            .expect("the archive has a central directory");

        bytes[offset + 24..offset + 28].copy_from_slice(&declared.to_le_bytes());
    }

    fn write_archive(dir: &Path, bytes: &[u8]) -> PathBuf {
        let path = dir.join("theme.zip");
        fs::write(&path, bytes).unwrap();

        path
    }

    /// Extracts `entries` with `limit` into a fresh scratch directory, returning
    /// `(extraction target, scratch directory)`.
    fn extract_ok(name: &str, entries: &[(&str, &[u8])], limit: u64) -> (PathBuf, PathBuf) {
        let dir = scratch(name);
        let zip_path = write_archive(&dir, &archive(entries));
        let target = dir.join("out");

        extract_with_limit(&zip_path, &target, limit).expect("extraction should have succeeded");

        (target, dir)
    }

    #[test]
    fn a_single_shared_root_directory_is_unwrapped() {
        let (target, dir) = extract_ok(
            "unwrap",
            &[("Theme/solar.json", b"[]"), ("Theme/jpg/1.jpg", b"image")],
            MAX_EXTRACTED_BYTES,
        );

        assert!(target.join("solar.json").is_file());
        assert!(target.join("jpg").join("1.jpg").is_file());
        assert!(!target.join("Theme").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_stray_top_level_file_keeps_the_root_directory() {
        // The daemon refuses a theme that sits one level down, so the layout has to
        // stay visible rather than being silently flattened or dropped.
        let (target, dir) = extract_ok(
            "stray",
            &[("Theme/solar.json", b"[]"), ("README.md", b"# notes")],
            MAX_EXTRACTED_BYTES,
        );

        assert!(target.join("Theme").join("solar.json").is_file());
        assert!(target.join("README.md").is_file());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn junk_entries_are_left_out_when_a_root_is_unwrapped() {
        let (target, dir) = extract_ok(
            "junk",
            &[
                ("Theme/solar.json", b"[]"),
                (".DS_Store", b"junk"),
                ("__MACOSX/._solar.json", b"junk"),
            ],
            MAX_EXTRACTED_BYTES,
        );

        assert!(target.join("solar.json").is_file());
        assert!(!target.join(".DS_Store").exists());
        assert!(!target.join("__MACOSX").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_nested_directory_is_created_on_demand() {
        let (target, dir) = extract_ok(
            "nested",
            &[("Theme/a/b/c/deep.txt", b"deep")],
            MAX_EXTRACTED_BYTES,
        );

        assert_eq!(
            fs::read_to_string(target.join("a").join("b").join("c").join("deep.txt")).unwrap(),
            "deep"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// An archive whose entries lie about their size is refused, and none of it is left
    /// behind.
    ///
    /// The regression this started as: summing the sizes the central directory declares
    /// and then handing the archive to the crate's extractor let an archive that
    /// declares 16 bytes and expands to 64 KiB fill the disk — the cap on written bytes
    /// is what stopped it. `copying_stops_exactly_at_the_limit` pins that cap directly,
    /// because `zip` 9 refuses a lying entry in its own reader ("File is larger than its
    /// declared uncompressed size"), so this extraction now fails before reaching it.
    #[test]
    fn an_archive_that_lies_about_its_sizes_is_refused_and_leaves_nothing() {
        let dir = scratch("lie");
        let mut bytes = archive(&[("Theme/big.bin", &[b'x'; 64 * 1024])]);
        lie_about_uncompressed_size(&mut bytes, 16);
        let zip_path = write_archive(&dir, &bytes);
        let target = dir.join("out");

        extract_with_limit(&zip_path, &target, 4096)
            .expect_err("an archive that lies about its sizes must not be extracted");

        assert!(
            !target.join("big.bin").is_file(),
            "the oversized file must not be left behind"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    /// A size that is over the budget before anything is decompressed is refused
    /// without writing, which is what the declared-size check is for.
    #[test]
    fn an_archive_larger_than_the_budget_is_refused_before_writing() {
        let dir = scratch("cap");
        let zip_path = write_archive(&dir, &archive(&[("Theme/big.bin", &[b'x'; 4096])]));
        let target = dir.join("out");

        let error = extract_with_limit(&zip_path, &target, 1024)
            .expect_err("the budget must reject the extraction")
            .to_string();

        assert!(
            error.contains("declares more than"),
            "the declared-size check has to reject it first: {error}"
        );
        assert!(!target.join("big.bin").is_file());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_archive_over_the_entry_limit_is_refused() {
        let dir = scratch("entries");
        let zip_path = dir.join("theme.zip");
        let mut writer = zip::ZipWriter::new(File::create(&zip_path).unwrap());
        let options = SimpleFileOptions::default();

        for index in 0..=MAX_ARCHIVE_ENTRIES {
            writer
                .start_file(format!("Theme/{index}.txt"), options)
                .unwrap();
        }
        writer.finish().unwrap();

        let result = extract_with_limit(&zip_path, &dir.join("out"), MAX_EXTRACTED_BYTES);

        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("too many entries"),
            "the entry limit must be what rejected it"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_non_zip_payload_is_refused() {
        let dir = scratch("notzip");
        let zip_path = dir.join("theme.zip");
        fs::write(&zip_path, b"<html>this is not a zip</html>").unwrap();

        let result = extract_with_limit(&zip_path, &dir.join("out"), MAX_EXTRACTED_BYTES);

        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_archive_is_refused() {
        let dir = scratch("missing");

        let result = extract_with_limit(
            &dir.join("absent.zip"),
            &dir.join("out"),
            MAX_EXTRACTED_BYTES,
        );

        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn copying_stops_exactly_at_the_limit() {
        let mut source = Cursor::new(vec![b'x'; 10]);
        let mut sink = Vec::new();
        assert_eq!(copy_bounded(&mut source, &mut sink, 10).unwrap(), 10);
        assert_eq!(
            sink.len(),
            10,
            "a source exactly at the limit is copied whole"
        );

        // One byte past it: the chunk that does not fit is refused before it is
        // written, and the failure names the limit that fired. This is the guarantee
        // the extraction rests on, independent of what the zip crate trusts.
        let mut source = Cursor::new(vec![b'x'; 11]);
        let mut sink = Vec::new();
        let error = copy_bounded(&mut source, &mut sink, 10)
            .expect_err("eleven bytes may not fit through a ten byte limit")
            .to_string();

        assert_eq!(sink.len(), 0, "the oversized chunk must not be written");
        assert!(error.contains("expands to more than"), "{error}");
    }

    #[test]
    fn an_entry_belonging_to_the_root_is_stripped_to_its_path() {
        let root = Path::new("Theme");

        assert_eq!(
            relative_destination(Path::new("Theme/jpg/1.jpg"), Some(root)),
            Some(PathBuf::from("jpg").join("1.jpg"))
        );
        assert_eq!(
            relative_destination(Path::new("jpg/1.jpg"), None),
            Some(PathBuf::from("jpg").join("1.jpg"))
        );
    }

    #[test]
    fn the_root_entry_itself_and_filtered_junk_have_no_destination() {
        let root = Path::new("Theme");

        assert_eq!(relative_destination(Path::new("Theme"), Some(root)), None);
        assert_eq!(
            relative_destination(Path::new(".DS_Store"), Some(root)),
            None
        );
    }

    #[test]
    fn removing_a_file_that_is_already_gone_is_not_an_error() {
        let dir = scratch("remove");
        let present = dir.join("present.zip");
        fs::write(&present, b"x").unwrap();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            assert!(remove_file(&present).await.is_ok());
            assert!(!present.exists());
            assert!(remove_file(&present).await.is_ok());
        });

        let _ = fs::remove_dir_all(&dir);
    }
}
