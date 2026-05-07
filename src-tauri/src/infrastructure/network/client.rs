use std::time::Duration;

use dwall::config::Network;
use reqwest::{Client, Proxy};

use crate::error::DwallSettingsResult;

/// Service for downloading files over HTTP
#[derive(Clone)]
pub(crate) struct HttpClient;

impl HttpClient {
    /// Create a new downloader instance
    pub fn create_client(network: Option<&Network>) -> DwallSettingsResult<Client> {
        let builder = reqwest::ClientBuilder::new().connect_timeout(Duration::from_secs(120));

        let builder = match network {
            Some(Network::Socks5 { host, port }) => {
                let proxy = Proxy::all(format!("socks5h://{host}:{port}")).map_err(|e| {
                    error!(error = ?e, "Failed to create SOCKS5 proxy: {host}:{port}");
                    e
                })?;

                info!("Using SOCKS5 proxy: {host}:{port}");
                builder.proxy(proxy)
            }
            _ => builder,
        };

        let client = builder.build().map_err(|e| {
            error!(error = %e, "Failed to create HTTP client");
            e
        })?;

        Ok(client)
    }
}
