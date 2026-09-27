//! Dwall custom theme manifest (`theme.toml`, schema 1).
//!
//! Produced by **Dwall Maker** (see its `docs/theme-format.md`). The manifest is
//! the single source of truth for a custom theme: metadata, appearance indices,
//! and the solar frames the daemon matches against.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::config::ImageFormat;
use crate::theme::ThemeError;
use crate::{DwallResult, SolarAngle};

/// Filename of the custom theme manifest.
pub const THEME_MANIFEST_FILENAME: &str = "theme.toml";

/// The only supported manifest schema version.
pub const SUPPORTED_SCHEMA: u32 = 1;

/// `theme.toml` top-level structure.
#[derive(Debug, Deserialize)]
pub struct ThemeManifest {
    pub schema: u32,
    pub theme: ThemeInfo,
    #[serde(default)]
    pub appearance: Option<Appearance>,
    #[serde(default)]
    pub solar: Option<SolarSection>,
    /// Maker-side provenance; ignored by the runtime.
    #[serde(default)]
    pub images: Vec<ThemeImage>,
}

/// Descriptive theme information (`[theme]`).
#[derive(Debug, Deserialize)]
pub struct ThemeInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: u32,
    pub created_at: u64,
    pub image_format: ImageFormat,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Light/dark image indices (`[appearance]`).
#[derive(Debug, Deserialize)]
pub struct Appearance {
    pub light: usize,
    pub dark: usize,
}

/// Solar-angle data (`[solar]`); tagged by `mode`.
#[derive(Debug, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum SolarSection {
    /// macOS HEIC embedded angles; frames carry no time.
    Native { frames: Vec<NativeFrame> },
    /// Authoring inputs (location/date/time) plus precomputed angles.
    Calculated {
        latitude: f64,
        longitude: f64,
        #[serde(default)]
        altitude: f64,
        reference_date: DateInput,
        frames: Vec<TimedFrame>,
    },
    /// HEIC time ratios; frames carry local time plus precomputed angles.
    Hours { frames: Vec<TimedFrame> },
}

/// A `native` solar frame.
#[derive(Debug, Deserialize)]
pub struct NativeFrame {
    pub index: usize,
    pub altitude: f64,
    pub azimuth: f64,
}

/// A `calculated`/`hours` solar frame.
#[derive(Debug, Deserialize)]
pub struct TimedFrame {
    pub index: usize,
    pub hour: u8,
    pub minute: u8,
    pub altitude: f64,
    pub azimuth: f64,
}

/// Reference date for `calculated` solar data.
#[derive(Debug, Deserialize)]
pub struct DateInput {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

/// Maker-side image provenance (`[[images]]`).
#[derive(Debug, Deserialize)]
pub struct ThemeImage {
    pub index: usize,
    #[serde(default)]
    pub original: Option<String>,
}

impl ThemeManifest {
    /// Whether `<theme_dir>/theme.toml` exists.
    pub fn exists(theme_dir: &Path) -> bool {
        theme_dir.join(THEME_MANIFEST_FILENAME).is_file()
    }

    /// Reads and schema-checks `<theme_dir>/theme.toml`.
    pub fn read(theme_dir: &Path) -> DwallResult<Self> {
        let path = theme_dir.join(THEME_MANIFEST_FILENAME);
        let content = fs::read_to_string(&path).map_err(|e| {
            error!(path = %path.display(), error = ?e, "Failed to read theme manifest");
            e
        })?;

        let manifest: Self = toml::from_str(&content).map_err(|e| {
            error!(path = %path.display(), error = ?e, "Failed to parse theme manifest");
            ThemeError::ThemeManifestParse(e.to_string())
        })?;

        if manifest.schema != SUPPORTED_SCHEMA {
            return Err(ThemeError::UnsupportedThemeSchema(manifest.schema).into());
        }

        Ok(manifest)
    }

    /// The theme's image format (drives the file extension, see 4.1).
    pub fn image_format(&self) -> &ImageFormat {
        &self.theme.image_format
    }

    /// The frames the runtime matches against, as [`SolarAngle`]s.
    pub fn solar_angles(&self) -> DwallResult<Vec<SolarAngle>> {
        let frames: Vec<(usize, f64, f64)> = match &self.solar {
            Some(SolarSection::Native { frames }) => frames
                .iter()
                .map(|f| (f.index, f.altitude, f.azimuth))
                .collect(),
            Some(SolarSection::Calculated { frames, .. })
            | Some(SolarSection::Hours { frames }) => frames
                .iter()
                .map(|f| (f.index, f.altitude, f.azimuth))
                .collect(),
            None => Vec::new(),
        };

        if frames.is_empty() {
            return Err(ThemeError::NoSolarFrames.into());
        }

        frames
            .into_iter()
            .map(|(index, altitude, azimuth)| {
                let index =
                    u8::try_from(index).map_err(|_| ThemeError::FrameIndexOutOfRange(index))?;
                Ok(SolarAngle::new(index, altitude, azimuth))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THEME_HEADER: &str = r#"
schema = 1

[theme]
id = "demo"
name = "Demo"
author = "alice"
version = 2
created_at = 1700000000
image_format = "jpeg"
"#;

    fn manifest_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("dwall_manifest_{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &Path, content: &str) {
        fs::write(dir.join(THEME_MANIFEST_FILENAME), content).unwrap();
    }

    #[test]
    fn parses_calculated_frames_and_format() {
        let dir = manifest_dir("calculated");
        write(
            &dir,
            &format!(
                r#"{THEME_HEADER}
[solar]
mode = "calculated"
latitude = 39.9
longitude = 116.4
reference_date = {{ year = 2024, month = 6, day = 21 }}

[[solar.frames]]
index = 0
hour = 6
minute = 0
altitude = -2.5
azimuth = 65.0

[[solar.frames]]
index = 1
hour = 12
minute = 0
altitude = 72.0
azimuth = 180.0
"#
            ),
        );

        let manifest = ThemeManifest::read(&dir).unwrap();
        assert_eq!(manifest.theme.name, "Demo");
        assert_eq!(manifest.image_format(), &ImageFormat::Jpeg);

        let angles = manifest.solar_angles().unwrap();
        assert_eq!(angles.len(), 2);
        assert_eq!(angles[0].index(), 0);
        assert!((angles[0].altitude() - (-2.5)).abs() < f64::EPSILON);
        assert!((angles[0].azimuth() - 65.0).abs() < f64::EPSILON);
        assert_eq!(angles[1].index(), 1);
        assert!((angles[1].azimuth() - 180.0).abs() < f64::EPSILON);
    }

    #[test]
    fn parses_native_frames() {
        let dir = manifest_dir("native");
        write(
            &dir,
            &format!(
                r#"{THEME_HEADER}
[solar]
mode = "native"

[[solar.frames]]
index = 3
altitude = 25.0
azimuth = 110.0
"#
            ),
        );

        let angles = ThemeManifest::read(&dir).unwrap().solar_angles().unwrap();
        assert_eq!(angles.len(), 1);
        assert_eq!(angles[0].index(), 3);
        assert!((angles[0].altitude() - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rejects_unsupported_schema() {
        let dir = manifest_dir("schema");
        write(&dir, &THEME_HEADER.replace("schema = 1", "schema = 2"));
        assert!(ThemeManifest::read(&dir).is_err());
    }

    #[test]
    fn rejects_missing_frames() {
        let dir = manifest_dir("noframes");
        write(&dir, THEME_HEADER);
        assert!(ThemeManifest::read(&dir).unwrap().solar_angles().is_err());
    }

    #[test]
    fn rejects_out_of_range_frame_index() {
        let dir = manifest_dir("bigindex");
        write(
            &dir,
            &format!(
                r#"{THEME_HEADER}
[solar]
mode = "native"

[[solar.frames]]
index = 300
altitude = 10.0
azimuth = 90.0
"#
            ),
        );
        assert!(ThemeManifest::read(&dir).unwrap().solar_angles().is_err());
    }
}
