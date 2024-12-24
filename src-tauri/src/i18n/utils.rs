use windows::{core::PWSTR, Win32::Globalization::GetUserPreferredUILanguages};

use crate::error::DwallSettingsResult;

pub(super) fn get_user_preferred_language() -> DwallSettingsResult<String> {
    let mut num_languages: u32 = 0;
    let mut buffer_size: u32 = 0;

    unsafe {
        GetUserPreferredUILanguages(
            0,                           // dwflags, 0 means no special options
            &mut num_languages,          // Number of languages (output)
            PWSTR(std::ptr::null_mut()), // No buffer yet, to get the size first
            &mut buffer_size,
        )?
    };

    let mut buffer: Vec<u16> = vec![0; buffer_size as usize];
    unsafe {
        GetUserPreferredUILanguages(
            0,                                      // dwflags, 0 means no special options
            &mut num_languages,                     // Number of languages (output)
            PWSTR(buffer.as_mut_ptr() as *mut u16), // The buffer to hold the languages
            &mut buffer_size,                       // Buffer size (output)
        )?;
    }

    let languages = buffer
        .split(|&c| c == 0)
        .filter_map(|chunk| String::from_utf16(chunk).ok())
        .collect::<Vec<String>>();

    let language = &languages[0];

    info!("Succesfully got user preferred language: {}", language);
    // Ok(language.to_string())
    Ok("en-US".to_string())
}
