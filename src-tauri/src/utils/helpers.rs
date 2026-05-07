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
pub async fn resolve_github_mirror_url(network: Option<&Network>, github_url: &str) -> String {
    match network {
        None | Some(Network::Socks5 { .. }) => github_url.to_owned(),
        Some(Network::GitHubMirrorTemplate(template)) => {
            if template.is_empty() {
                return github_url.to_owned();
            }

            let prefix = "https://github.com/";
            if !github_url.starts_with(prefix) {
                return github_url.to_owned();
            }

            let remaining = &github_url[prefix.len()..];
            let parts: Vec<&str> = remaining.split('/').collect();

            if parts.len() < 2 {
                return github_url.to_owned();
            }

            let owner = parts[0];
            let repo = parts[1];

            // Format 1: releases/download/{version}/{asset}
            if parts.len() >= 6 && parts[2] == "releases" && parts[3] == "download" {
                let version = parts[4];
                let asset = parts[5..].join("/");
                return template
                    .replace("<owner>", owner)
                    .replace("<repo>", repo)
                    .replace("<version>", version)
                    .replace("<asset>", &asset);
            }

            // Format 2: releases/latest/download/{asset}
            if parts.len() >= 6
                && parts[2] == "releases"
                && parts[3] == "latest"
                && parts[4] == "download"
            {
                let asset = parts[5..].join("/");
                let api_url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    owner, repo
                );

                if let Ok(version) = fetch_latest_release_tag(&api_url).await {
                    return template
                        .replace("<owner>", owner)
                        .replace("<repo>", repo)
                        .replace("<version>", &version)
                        .replace("<asset>", &asset);
                }

                return github_url.to_owned();
            }

            github_url.to_owned()
        }
    }
}

async fn fetch_latest_release_tag(api_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent("resolve-mirror/1.0")
        .build()?;

    let json: serde_json::Value = client
        .get(api_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    json["tag_name"]
        .as_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| "tag_name not found".into())
}
