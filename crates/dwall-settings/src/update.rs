//! Which release an update check looks at.
//!
//! A stable build asks for `releases/latest`, an endpoint GitHub keeps free of
//! pre-releases, so it can only ever be offered a stable version. A pre-release build
//! cannot use that: it has to list the releases and pick the newest version that is
//! really newer than the one running. A stable release wins over a pre-release of the
//! same or a newer base (`0.3.0` over `0.3.0-alpha.1`), which is what moves a tester
//! back onto the stable channel as soon as one exists; only when no stable release is
//! newer does a newer pre-release get offered. Nothing is ever selected that is not
//! strictly newer, so a build can never be pushed backwards.

use semver::Version;
use serde::Deserialize;

use crate::error::{DwallSettingsError, DwallSettingsResult};
use crate::network::HttpClient;

/// Updater manifest attached to every release; without it a release cannot be
/// installed by the updater, so it is not a candidate.
const UPDATER_MANIFEST: &str = "latest.json";

/// Newest stable release, as the updater sees it.
///
/// GitHub excludes pre-releases and drafts from this endpoint, which is exactly the
/// guarantee a stable build wants.
pub const LATEST_ENDPOINT: &str =
    "https://github.com/dwall-rs/dwall/releases/latest/download/latest.json";

/// Release list used by pre-release builds.
const RELEASES_API: &str = "https://api.github.com/repos/dwall-rs/dwall/releases?per_page=30";

/// Upper bound for the release list.
///
/// Thirty releases are a few hundred kilobytes of JSON; anything far beyond that is
/// not this endpoint and should not be buffered.
const RELEASES_API_MAX_BYTES: usize = 4 * 1024 * 1024;

/// One release, reduced to what choosing an update needs.
#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}

/// One downloadable file of a release.
#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

impl Release {
    /// Version of the release, when its tag is one (`v0.3.0-alpha.1`).
    ///
    /// Tags that are not versions (the `nightly` release, for instance) return `None`
    /// rather than being guessed at.
    pub fn version(&self) -> Option<Version> {
        self.tag_name
            .strip_prefix('v')
            .unwrap_or(&self.tag_name)
            .parse()
            .ok()
    }

    /// URL of the release's updater manifest, when it published one.
    pub fn updater_manifest(&self) -> Option<&str> {
        self.assets
            .iter()
            .find(|asset| asset.name == UPDATER_MANIFEST)
            .map(|asset| asset.browser_download_url.as_str())
    }
}

/// Picks the release a pre-release build should update to.
///
/// `None` means "nothing newer": every entry is a draft, not a version, missing its
/// updater manifest, or not newer than `current`.
pub fn select_release<'a>(current: &Version, releases: &'a [Release]) -> Option<&'a Release> {
    let mut stable: Option<(Version, &Release)> = None;
    let mut prerelease: Option<(Version, &Release)> = None;

    for release in releases {
        if release.draft || release.updater_manifest().is_none() {
            continue;
        }

        let Some(version) = release.version() else {
            continue;
        };
        if version <= *current {
            continue;
        }

        let slot = if release.prerelease {
            &mut prerelease
        } else {
            &mut stable
        };
        if slot.as_ref().is_none_or(|(best, _)| version > *best) {
            *slot = Some((version, release));
        }
    }

    stable.or(prerelease).map(|(_, release)| release)
}

/// Updater manifest URL of the release a pre-release build should move to.
///
/// `Ok(None)` means the build is up to date; an error is reported to the caller rather
/// than being turned into "up to date", because a rate-limited or unreachable API is
/// not evidence that no update exists.
pub async fn prerelease_endpoint(
    http_client: &HttpClient,
    current: &Version,
) -> DwallSettingsResult<Option<String>> {
    let body = http_client
        .get_text(RELEASES_API, RELEASES_API_MAX_BYTES)
        .await?;

    let releases: Vec<Release> = serde_json::from_str(&body).map_err(|e| {
        error!(error = %e, "Failed to parse the release list");
        DwallSettingsError::Other(format!("release list is not valid JSON: {e}"))
    })?;

    let endpoint = select_release(current, &releases)
        .and_then(Release::updater_manifest)
        .map(str::to_owned);

    match &endpoint {
        Some(url) => {
            info!(version = %current, endpoint = %url, "Newer release found for this pre-release")
        }
        None => info!(version = %current, "No newer release for this pre-release"),
    }

    Ok(endpoint)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, prerelease: bool) -> Release {
        Release {
            tag_name: tag.to_string(),
            prerelease,
            draft: false,
            assets: vec![Asset {
                name: UPDATER_MANIFEST.to_string(),
                browser_download_url: format!("https://example.test/{tag}/{UPDATER_MANIFEST}"),
            }],
        }
    }

    fn version(text: &str) -> Version {
        Version::parse(text).expect("test version")
    }

    fn selected(current: &str, releases: &[Release]) -> Option<String> {
        select_release(&version(current), releases)
            .and_then(Release::version)
            .map(|v| v.to_string())
    }

    #[test]
    fn a_newer_prerelease_is_offered() {
        let releases = [
            release("v0.3.0-alpha.2", true),
            release("v0.3.0-alpha.1", true),
            release("v0.3.0-alpha.0", true),
            release("v0.2.5", false),
        ];

        assert_eq!(
            selected("0.3.0-alpha.0", &releases),
            Some("0.3.0-alpha.2".to_string())
        );
    }

    #[test]
    fn a_stable_release_wins_over_a_newer_prerelease() {
        let releases = [
            release("v0.4.0-alpha.0", true),
            release("v0.3.1", false),
            release("v0.3.0", false),
        ];

        // Leaving the pre-release channel for a released build is the point, even when
        // a pre-release of a higher base exists.
        assert_eq!(
            selected("0.3.0-alpha.2", &releases),
            Some("0.3.1".to_string())
        );
    }

    #[test]
    fn the_newest_stable_wins_when_several_exist() {
        let releases = [release("v0.3.0", false), release("v0.3.1", false)];

        assert_eq!(
            selected("0.3.0-alpha.0", &releases),
            Some("0.3.1".to_string())
        );
    }

    #[test]
    fn nothing_is_offered_when_nothing_is_newer() {
        let releases = [
            release("v0.3.0-alpha.2", true),
            release("v0.2.5", false),
            release("v0.2.4", false),
        ];

        assert_eq!(selected("0.3.0-alpha.2", &releases), None);
        // A pre-release is never a downgrade target for a newer stable build either.
        assert_eq!(selected("0.3.0", &releases), None);
    }

    #[test]
    fn drafts_tags_that_are_not_versions_and_releases_without_a_manifest_are_ignored() {
        let mut draft = release("v0.5.0", false);
        draft.draft = true;
        let mut no_manifest = release("v0.4.0", false);
        no_manifest.assets.clear();
        let releases = [
            draft,
            no_manifest,
            release("nightly", true),
            release("v0.3.0-alpha.3", true),
        ];

        assert_eq!(
            selected("0.3.0-alpha.2", &releases),
            Some("0.3.0-alpha.3".to_string())
        );
    }

    /// Parses a real answer from the releases API, so the field names stay the ones
    /// GitHub actually sends (trimmed to the four newest releases).
    #[test]
    fn a_real_release_list_is_parsed() {
        let releases: Vec<Release> =
            serde_json::from_str(REAL_RELEASE_LIST).expect("fixture parses");

        assert_eq!(releases.len(), 4);
        assert_eq!(
            selected("0.3.0-alpha.0", &releases),
            Some("0.3.0-alpha.2".to_string())
        );
        assert_eq!(selected("0.3.0-alpha.2", &releases), None);
        assert_eq!(
            releases[0].updater_manifest(),
            Some("https://github.com/dwall-rs/dwall/releases/download/v0.3.0-alpha.2/latest.json")
        );
    }

    /// Captured from `GET /repos/dwall-rs/dwall/releases?per_page=4`, assets trimmed to
    /// the updater manifest except for the newest release.
    const REAL_RELEASE_LIST: &str = r#"[
      {
        "url": "https://api.github.com/repos/dwall-rs/dwall/releases/262942856",
        "tag_name": "v0.3.0-alpha.2",
        "name": "v0.3.0-alpha.2",
        "draft": false,
        "prerelease": true,
        "created_at": "2026-10-15T09:12:04Z",
        "published_at": "2026-10-15T09:31:55Z",
        "assets": [
          {
            "name": "Dwall.Settings_0.3.0-alpha.2_x64-setup.exe",
            "browser_download_url": "https://github.com/dwall-rs/dwall/releases/download/v0.3.0-alpha.2/Dwall.Settings_0.3.0-alpha.2_x64-setup.exe"
          },
          {
            "name": "latest.json",
            "browser_download_url": "https://github.com/dwall-rs/dwall/releases/download/v0.3.0-alpha.2/latest.json"
          }
        ]
      },
      {
        "tag_name": "v0.3.0-alpha.1",
        "draft": false,
        "prerelease": true,
        "assets": [
          {
            "name": "latest.json",
            "browser_download_url": "https://github.com/dwall-rs/dwall/releases/download/v0.3.0-alpha.1/latest.json"
          }
        ]
      },
      {
        "tag_name": "v0.3.0-alpha.0",
        "draft": false,
        "prerelease": true,
        "assets": [
          {
            "name": "latest.json",
            "browser_download_url": "https://github.com/dwall-rs/dwall/releases/download/v0.3.0-alpha.0/latest.json"
          }
        ]
      },
      {
        "tag_name": "v0.2.5",
        "draft": false,
        "prerelease": false,
        "assets": [
          {
            "name": "latest.json",
            "browser_download_url": "https://github.com/dwall-rs/dwall/releases/download/v0.2.5/latest.json"
          }
        ]
      }
    ]"#;
}
