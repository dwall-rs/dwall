//! GitHub mirror URL resolution.

use dwall::config::Network;

/// Resolves GitHub URLs to mirror URLs based on network configuration
pub struct MirrorUrlResolver;

impl MirrorUrlResolver {
    /// Resolve a GitHub URL using the configured mirror template.
    ///
    /// Purely local: the release route is filled in with the `latest` alias, so no
    /// request leaves the machine and a usable mirror is never bypassed in favour of
    /// the original `github.com` URL.
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
                    // `latest` is a real segment of GitHub's download route and every
                    // proxy that mirrors it passes the alias through, so the template can
                    // be filled directly. Asking `api.github.com` for the tag made the
                    // check depend on the very host a mirror is configured to avoid, and
                    // its failure fell back to the original `github.com` URL.
                    return apply_template(
                        template,
                        owner,
                        repo,
                        Some("latest"),
                        &asset,
                        github_url,
                    );
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

    /// A tagged release asset — the shape the update check uses for a pre-release —
    /// is rewritten without asking the API for the tag.
    #[tokio::test]
    async fn resolve_rewrites_a_tagged_release_asset() {
        let network =
            mirror("https://ghproxy.cc/<owner>/<repo>/releases/download/<version>/<asset>");

        assert_eq!(
            MirrorUrlResolver::resolve(
                network.as_ref(),
                "https://github.com/dwall-rs/dwall/releases/download/v0.3.0-alpha.2/latest.json"
            )
            .await,
            "https://ghproxy.cc/dwall-rs/dwall/releases/download/v0.3.0-alpha.2/latest.json"
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

    /// The stable channel's manifest uses the `latest` alias. It must be rewritten
    /// through the template without a request, or the mirror is bypassed for the one
    /// check every stable build performs.
    #[tokio::test]
    async fn resolve_rewrites_a_latest_release_asset_without_the_api() {
        let network =
            mirror("https://ghproxy.cc/<owner>/<repo>/releases/download/<version>/<asset>");

        assert_eq!(
            MirrorUrlResolver::resolve(
                network.as_ref(),
                "https://github.com/dwall-rs/dwall/releases/latest/download/latest.json",
            )
            .await,
            "https://ghproxy.cc/dwall-rs/dwall/releases/download/latest/latest.json"
        );
    }

    /// A host-swap template cannot express a release asset; the original URL is kept
    /// rather than a mirror URL that is guaranteed to 404.
    #[tokio::test]
    async fn resolve_keeps_the_original_for_a_template_without_an_asset() {
        let network = mirror("https://kkgithub.com/<owner>/<repo>");
        let url = "https://github.com/dwall-rs/dwall/releases/latest/download/latest.json";

        assert_eq!(MirrorUrlResolver::resolve(network.as_ref(), url).await, url);
    }

    // ── resolve: the pass-through cases ─────────────────────────────────────

    /// No network setting, a SOCKS5 proxy, and a URL that is not GitHub: all three
    /// have to leave the URL exactly as it was, because only a mirror template
    /// rewrites anything.
    #[tokio::test]
    async fn resolve_passes_none_a_proxy_and_non_github_urls_through_unchanged() {
        let github = "https://github.com/dwall-rs/dwall/releases/download/v0.3.0/latest.json";
        let elsewhere = "https://example.com/releases/download/v0.3.0/latest.json";

        assert_eq!(MirrorUrlResolver::resolve(None, github).await, github);

        let proxy = Some(Network::Socks5 {
            host: "127.0.0.1".to_string(),
            port: 1080,
        });
        assert_eq!(
            MirrorUrlResolver::resolve(proxy.as_ref(), github).await,
            github
        );

        let template = mirror("https://ghproxy.cc/https://github.com/<owner>/<repo>");
        assert_eq!(
            MirrorUrlResolver::resolve(template.as_ref(), elsewhere).await,
            elsewhere,
            "only github.com URLs can be rewritten through a mirror"
        );
    }

    // ── resolve_raw: the degenerate cases ───────────────────────────────────

    /// A template with only one of the two placeholders is not usable and the URL is
    /// kept, rather than being half-substituted into a URL that 404s.
    #[test]
    fn resolve_raw_keeps_the_url_when_a_placeholder_is_missing() {
        let url = "https://github.com/dwall-rs/dwall-assets/raw/refs/heads/main/thumbnails/1.avif";

        for template in [
            "https://ghproxy.cc/https://github.com/<owner>",
            "https://ghproxy.cc/https://github.com/<repo>",
        ] {
            assert_eq!(
                MirrorUrlResolver::resolve_raw(mirror(template).as_ref(), url),
                url,
                "{template} is missing a placeholder"
            );
        }
    }

    /// A GitHub URL that is not a raw file has no path to append, so the base the
    /// template builds is the whole answer.
    #[test]
    fn resolve_raw_without_a_raw_path_keeps_the_mirrored_base() {
        let network = mirror("https://ghproxy.cc/https://github.com/<owner>/<repo>");

        assert_eq!(
            MirrorUrlResolver::resolve_raw(
                network.as_ref(),
                "https://github.com/dwall-rs/dwall-assets",
            ),
            "https://ghproxy.cc/https://github.com/dwall-rs/dwall-assets"
        );
    }
}
