//! Whether an executable gets a console window when Windows starts it.
//!
//! Release builds of the daemon are GUI-subsystem executables, so the `Run` key
//! never gives them a console; a debug build is a console application and would
//! show one at every logon. Only the PE subsystem recorded in the image answers
//! that: a console build started from a terminal does not own the console it is
//! attached to, and a GUI build started from a terminal is attached to one too,
//! so asking about the current process would get both cases backwards.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// `IMAGE_SUBSYSTEM_WINDOWS_GUI`: Windows gives this image no console.
const WINDOWS_GUI: u16 = 2;
/// `IMAGE_SUBSYSTEM_WINDOWS_CUI`: Windows gives this image a console window.
const WINDOWS_CUI: u16 = 3;
/// Offset of the `e_lfanew` field in the DOS header.
const PE_OFFSET_FIELD: u64 = 0x3C;
/// Offset of `Subsystem` in the optional header (PE32 and PE32+ agree here).
const SUBSYSTEM_FIELD: u64 = 0x5C;
/// `IMAGE_NT_SIGNATURE`, i.e. `PE\0\0`.
const PE_SIGNATURE: [u8; 4] = *b"PE\0\0";

/// Whether the running executable is given a console window by Windows.
///
/// `None` when the image cannot be inspected, so a caller can stay quiet rather
/// than report a problem it cannot prove.
pub fn executable_opens_console() -> Option<bool> {
    opens_console(&std::env::current_exe().ok()?)
}

/// Whether the PE image at `path` is given a console window by Windows.
fn opens_console(path: &Path) -> Option<bool> {
    let mut file = File::open(path).ok()?;

    let mut offset = [0u8; 4];
    file.seek(SeekFrom::Start(PE_OFFSET_FIELD)).ok()?;
    file.read_exact(&mut offset).ok()?;
    let nt_header = u64::from(u32::from_le_bytes(offset));

    let mut signature = [0u8; 4];
    file.seek(SeekFrom::Start(nt_header)).ok()?;
    file.read_exact(&mut signature).ok()?;
    if signature != PE_SIGNATURE {
        return None;
    }

    let mut subsystem = [0u8; 2];
    file.seek(SeekFrom::Start(nt_header + SUBSYSTEM_FIELD))
        .ok()?;
    file.read_exact(&mut subsystem).ok()?;

    match u16::from_le_bytes(subsystem) {
        WINDOWS_CUI => Some(true),
        WINDOWS_GUI => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The test binary is a console application (nothing sets
    /// `windows_subsystem` for it), which pins the parser against a real image.
    #[test]
    fn the_test_binary_is_a_console_image() {
        assert_eq!(executable_opens_console(), Some(true));
    }

    /// A source file is not a PE image, so the answer stays unknown instead of
    /// being guessed.
    #[test]
    fn a_file_that_is_not_a_pe_image_is_unknown() {
        assert_eq!(opens_console(Path::new(file!())), None);
    }

    /// Neither is a file that does not exist.
    #[test]
    fn a_missing_file_is_unknown() {
        assert_eq!(opens_console(Path::new("no-such-image.exe")), None);
    }
}
