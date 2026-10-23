//! Theme download URL builder.

use std::fmt::Write as _;

/// Builds download URLs for themes from GitHub releases
pub struct ThemeDownloadUrlBuilder;

impl ThemeDownloadUrlBuilder {
    /// Build the download URL for a theme
    pub fn build(theme_id: &str) -> String {
        format!(
            "https://github.com/dwall-rs/dwall-assets/releases/download/themes/{}.zip",
            encode_segment(&theme_id.replace(' ', "."))
        )
    }
}

/// Percent-encodes one path segment, leaving the unreserved characters alone.
///
/// The release assets rename a theme id's spaces to dots, so that substitution has
/// to stay; everything else is escaped, because a `#` would cut the URL off at a
/// fragment, a `?` would start a query and a `%` would begin an escape sequence —
/// each of them fetching something other than the asset the id names.
fn encode_segment(segment: &str) -> String {
    let mut encoded = String::with_capacity(segment.len());

    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_theme_id_is_passed_through() {
        assert_eq!(
            ThemeDownloadUrlBuilder::build("Sunrise"),
            "https://github.com/dwall-rs/dwall-assets/releases/download/themes/Sunrise.zip"
        );
    }

    /// The assets rename spaces to dots, so the URL has to rename them too.
    #[test]
    fn spaces_become_dots_like_the_asset_names_do() {
        assert_eq!(
            ThemeDownloadUrlBuilder::build("Sun Rise"),
            "https://github.com/dwall-rs/dwall-assets/releases/download/themes/Sun.Rise.zip"
        );
    }

    /// A `#` would truncate the URL at a fragment, so it has to be escaped rather than
    /// handed to the server as written.
    #[test]
    fn characters_that_carry_url_meaning_are_escaped() {
        assert_eq!(
            ThemeDownloadUrlBuilder::build("C# & F#"),
            "https://github.com/dwall-rs/dwall-assets/releases/download/themes/C%23.%26.F%23.zip"
        );
    }

    #[test]
    fn non_ascii_theme_ids_are_encoded_byte_by_byte() {
        assert_eq!(
            ThemeDownloadUrlBuilder::build("昼与夜"),
            "https://github.com/dwall-rs/dwall-assets/releases/download/themes/%E6%98%BC%E4%B8%8E%E5%A4%9C.zip"
        );
    }
}
