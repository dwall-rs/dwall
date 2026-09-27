//! URL mirroring commands.

use dwall::config::Network;

use crate::network::MirrorUrlResolver;

/// Resolve a raw GitHub file URL (e.g. a theme thumbnail) through the configured
/// mirror template. Returns the URL unchanged when no mirror is configured.
#[tauri::command]
pub fn mirror_url(url: String, network: Option<Network>) -> String {
    MirrorUrlResolver::resolve_raw(network.as_ref(), &url)
}
