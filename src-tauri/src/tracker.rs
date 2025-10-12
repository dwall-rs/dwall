pub async fn track() {
    let client = reqwest::Client::new();
    if let Err(e) = client
        .post(format!("{}?platform=windows", env!("DWALL_TRACKER_URL")))
        .send()
        .await
    {
        error!("Failed to track: {}", e);
    }
}
