//! Theme metadata types

use std::path::PathBuf;

use dwall::config::ImageFormat;
use serde::{Deserialize, Serialize};

/// Customized theme information
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomizedTheme {
    pub id: String,
    pub directory: PathBuf,
    /// 先从 thumbnails 获取，如果不存在则从 images 获取
    pub thumbnails: Vec<PathBuf>,
    #[serde(flatten)]
    pub metadata: CustomizedThemeMetadata,
}

/// Metadata for a customized theme
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomizedThemeMetadata {
    pub image_format: ImageFormat,
    pub theme_name: String,
    pub author: String,
    pub version: u16,
}
