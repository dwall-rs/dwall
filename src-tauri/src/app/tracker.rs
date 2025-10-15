//! Application tracker module
//!
//! This module contains functionality for tracking application usage.

pub async fn track() {
    let client = reqwest::Client::new();
    if let Err(e) = client
        .post("https://app.thepoy.cc/api/track/dwall?platform=windows")
        .send()
        .await
    {
        error!("Failed to track: {}", e);
    }
}
