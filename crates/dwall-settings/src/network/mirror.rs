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
                    return apply_template(
                        template,
                        owner,
                        repo,
                        Some(version),
                        &asset,
                        github_url,
                    );
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
                        return apply_template(
                            template,
                            owner,
                            repo,
                            Some(&version),
                            &asset,
                            github_url,
                        );
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

        if !template_has_placeholders(template) {
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

/// Whether the template carries the placeholders needed to build a real URL.
///
/// A bare host (a legacy value someone pasted from an older guide) would otherwise
/// expand to a URL that is guaranteed to 404; falling back to the original GitHub
/// link is more useful and keeps the failure visible.
fn template_has_placeholders(template: &str) -> bool {
    template.contains("<owner>") && template.contains("<repo>")
}

/// Substitutes the release placeholders, falling back to the original URL when the
/// template cannot express the asset.
fn apply_template(
    template: &str,
    owner: &str,
    repo: &str,
    version: Option<&str>,
    asset: &str,
    fallback: &str,
) -> String {
    if !template_has_placeholders(template) || !template.contains("<asset>") {
        return fallback.to_owned();
    }

    let mut resolved = template
        .replace("<owner>", owner)
        .replace("<repo>", repo)
        .replace("<asset>", asset);

    if let Some(version) = version {
        resolved = resolved.replace("<version>", version);
    }

    resolved
}

async fn fetch_latest_release_tag(api_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Bounded: this runs while the user waits for the update check, and the GitHub
    // API is unreachable in the very regions the mirror template exists for.
    let client = reqwest::Client::builder()
        .user_agent("resolve-mirror/1.0")
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(10))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mirror(template: &str) -> Option<Network> {
        Some(Network::GitHubMirrorTemplate(template.to_string()))
    }

    #[test]
    fn resolve_raw_rewrites_a_thumbnail_url() {
        let network = mirror("https://ghproxy.cc/https://github.com/<owner>/<repo>");

        assert_eq!(
            MirrorUrlResolver::resolve_raw(
                network.as_ref(),
                "https://github.com/dwall-rs/dwall-assets/raw/refs/heads/main/thumbnails/BigSur/1.avif",
            ),
            "https://ghproxy.cc/https://github.com/dwall-rs/dwall-assets/raw/refs/heads/main/thumbnails/BigSur/1.avif"
        );
    }

    #[test]
    fn resolve_raw_rewrites_a_host_swapping_template() {
        let network = mirror("https://kkgithub.com/<owner>/<repo>");

        assert_eq!(
            MirrorUrlResolver::resolve_raw(
                network.as_ref(),
                "https://github.com/dwall-rs/dwall-assets/raw/refs/heads/main/thumbnails/BigSur/1.avif",
            ),
            "https://kkgithub.com/dwall-rs/dwall-assets/raw/refs/heads/main/thumbnails/BigSur/1.avif"
        );
    }

    #[test]
    fn resolve_raw_leaves_non_github_and_empty_templates_alone() {
        let url = "https://example.com/thumbnails/1.avif";

        assert_eq!(MirrorUrlResolver::resolve_raw(None, url), url);
        assert_eq!(
            MirrorUrlResolver::resolve_raw(mirror("").as_ref(), url),
            url
        );
        assert_eq!(
            MirrorUrlResolver::resolve_raw(
                mirror("https://ghproxy.cc/https://github.com/<owner>/<repo>").as_ref(),
                url
            ),
            url
        );
    }

    #[test]
    fn resolve_raw_ignores_templates_without_placeholders() {
        // A legacy value such as the bare host cannot produce a usable URL; the
        // original URL must be kept rather than a guaranteed 404.
        let url = "https://github.com/dwall-rs/dwall-assets/raw/refs/heads/main/thumbnails/1.avif";

        assert_eq!(
            MirrorUrlResolver::resolve_raw(mirror("https://ghproxy.cc").as_ref(), url),
            url
        );
    }

    #[test]
    fn apply_template_keeps_the_release_url_shape() {
        let template = "https://ghproxy.cc/https://github.com/<owner>/<repo>/releases/download/<version>/<asset>";
        let url = "https://github.com/dwall-rs/dwall-assets/releases/download/themes/Big.Sur.zip";

        assert_eq!(
            MirrorUrlResolver::resolve_raw(None, url),
            url,
            "no mirror configured"
        );
        assert_eq!(
            apply_template(
                template,
                "dwall-rs",
                "dwall-assets",
                Some("themes"),
                "Big.Sur.zip",
                url
            ),
            template
                .replace("<owner>", "dwall-rs")
                .replace("<repo>", "dwall-assets")
                .replace("<version>", "themes")
                .replace("<asset>", "Big.Sur.zip")
        );
    }

    #[test]
    fn apply_template_falls_back_without_an_asset_placeholder() {
        let url = "https://github.com/dwall-rs/dwall-assets/releases/download/themes/Big.Sur.zip";

        assert_eq!(
            apply_template(
                "https://ghproxy.cc/https://github.com/<owner>/<repo>",
                "dwall-rs",
                "dwall-assets",
                Some("themes"),
                "Big.Sur.zip",
                url,
            ),
            url
        );
    }
}
