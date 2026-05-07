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

use dwall::config::Network;

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
        match self.inner.network() {
            None | Some(Network::Socks5 { .. }) => github_url.to_owned(),
            Some(Network::GitHubMirrorTemplate(template)) => {
                if template.is_empty() {
                    return github_url.to_owned();
                }

                // Parse GitHub URL: https://github.com/{owner}/{repo}/releases/download/{version}/{asset}
                let prefix = "https://github.com/";
                if !github_url.starts_with(prefix) {
                    return github_url.to_owned();
                }

                let remaining = &github_url[prefix.len()..];
                let parts: Vec<&str> = remaining.split('/').collect();

                // Expected format: {owner}/{repo}/releases/download/{version}/{asset}
                if parts.len() >= 5 && parts[2] == "releases" && parts[3] == "download" {
                    let owner = parts[0];
                    let repo = parts[1];
                    let version = parts[4];
                    // Asset might contain slashes, so join the remaining parts
                    let asset = parts[5..].join("/");

                    template
                        .replace("<owner>", owner)
                        .replace("<repo>", repo)
                        .replace("<version>", version)
                        .replace("<asset>", &asset)
                } else {
                    github_url.to_owned()
                }
            }
        }
    }

    /// Returns the themes directory path
    pub fn themes_directory(&self) -> &Path {
        self.inner.themes_directory()
    }
}
