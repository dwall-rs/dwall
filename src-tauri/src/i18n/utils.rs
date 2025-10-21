use windows::{core::PWSTR, Win32::Globalization::GetUserPreferredUILanguages};

use crate::error::DwallSettingsResult;

pub(super) fn get_user_preferred_language() -> DwallSettingsResult<String> {
    trace!("Entering get_user_preferred_language function");

    let mut num_languages: u32 = 0;
    let mut buffer_size: u32 = 0;

    unsafe {
        GetUserPreferredUILanguages(
            0,                  // dwflags, 0 means no special options
            &mut num_languages, // Number of languages (output)
            None,               // No buffer yet, to get the size first
            &mut buffer_size,
        )?
    };

    debug!(buffer_size, "Initial buffer size obtained");
    let mut buffer: Vec<u16> = vec![0; buffer_size as usize];
    unsafe {
        GetUserPreferredUILanguages(
            0,                                // dwflags, 0 means no special options
            &mut num_languages,               // Number of languages (output)
            Some(PWSTR(buffer.as_mut_ptr())), // The buffer to hold the languages
            &mut buffer_size,                 // Buffer size (output)
        )?;
    }

    trace!(num_languages, "Number of user preferred languages obtained");

    let languages = buffer
        .split(|&c| c == 0)
        .filter_map(|chunk| String::from_utf16(chunk).ok())
        .collect::<Vec<String>>();

    if languages.is_empty() {
        warn!("No user preferred languages found, falling back to en-US");
        return Ok("en-US".to_string());
    }

    let language = &languages[0];

    debug!(language, "Successfully got user preferred language");
    Ok(language.to_string())
}
