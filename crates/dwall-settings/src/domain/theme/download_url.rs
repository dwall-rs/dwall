//! Theme download URL builder (domain knowledge: GitHub release structure)

/// Builds download URLs for themes from GitHub releases
pub struct ThemeDownloadUrlBuilder;

impl ThemeDownloadUrlBuilder {
    /// Build the download URL for a theme
    pub fn build(theme_id: &str) -> String {
        format!(
            "https://github.com/dwall-rs/dwall-assets/releases/download/themes/{}.zip",
            theme_id.replace(' ', ".")
        )
    }
}
