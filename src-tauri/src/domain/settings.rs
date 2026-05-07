//! Settings domain module
//!
//! This module contains the core business logic related to application settings.
//!
//! # Overview
//!
//! The settings domain handles configuration management, including loading,
//! saving, and validating application settings.
//!
//! # Key Components
//!
//! - Settings validation logic
//! - Configuration file management
//! - Settings migration (when needed)

// Placeholder for settings-related business logic
// Will be populated as we refactor the existing code

use std::path::Path;

use crate::utils::helpers::resolve_github_mirror_url;

pub struct Config {
    inner: dwall::config::Config,
}

impl Config {
    pub fn new(inner: dwall::config::Config) -> Self {
        Self { inner }
    }

    /// Resolves a GitHub asset URL to a mirrored URL if a mirror template is configured
    ///
    /// This method transforms GitHub release URLs using the configured mirror template,
    /// replacing placeholders like `<owner>`, `<repo>`, `<version>`, and `<asset>`.
    pub fn resolve_github_mirror_url(&self, github_url: &str) -> String {
        resolve_github_mirror_url(self.inner.network(), github_url)
    }

    /// Returns the themes directory path
    pub fn themes_directory(&self) -> &Path {
        self.inner.themes_directory()
    }
}
