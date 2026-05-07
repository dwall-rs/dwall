//! Helper functions
//!
//! This module contains various helper functions used throughout the application.

use dwall::config::Network;

/// Creates directories if they don't exist
pub async fn create_dir_if_missing<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        tokio::fs::create_dir_all(path).await
    } else {
        Ok(())
    }
}

/// Resolves a GitHub mirror URL based on the network configuration and the given URL.
pub fn resolve_github_mirror_url(network: Option<&Network>, github_url: &str) -> String {
    match network {
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
