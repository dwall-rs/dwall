//! GitHub mirror URL resolution.

use dwall::config::Network;

/// Resolves GitHub URLs to mirror URLs based on network configuration
pub struct MirrorUrlResolver;

impl MirrorUrlResolver {
    /// Resolve a GitHub URL using the configured mirror template
    pub async fn resolve(network: Option<&Network>, github_url: &str) -> String {
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

                if parts.len() >= 6 && parts[2] == "releases" && parts[3] == "download" {
                    let version = parts[4];
                    let asset = parts[5..].join("/");
                    return template
                        .replace("<owner>", owner)
                        .replace("<repo>", repo)
                        .replace("<version>", version)
                        .replace("<asset>", &asset);
                }

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

    /// Resolve a raw GitHub file URL (e.g. a theme thumbnail) through the mirror
    /// template. Unlike [`resolve`](Self::resolve), this is synchronous and does
    /// not require a network round-trip.
    pub fn resolve_raw(network: Option<&Network>, github_url: &str) -> String {
        let Some(Network::GitHubMirrorTemplate(template)) = network else {
            return github_url.to_owned();
        };

        let prefix = "https://github.com/";
        if template.is_empty() || !github_url.starts_with(prefix) {
            return github_url.to_owned();
        }

        let mut parts = github_url[prefix.len()..].split('/');
        let (Some(owner), Some(repo)) = (parts.next(), parts.next()) else {
            return github_url.to_owned();
        };

        let base_end = template
            .find("<repo>")
            .map_or(template.len(), |i| i + "<repo>".len());
        let base = template[..base_end]
            .replace("<owner>", owner)
            .replace("<repo>", repo);

        let raw_path = github_url.find("/raw/").map_or("", |i| &github_url[i..]);

        format!("{base}{raw_path}")
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
