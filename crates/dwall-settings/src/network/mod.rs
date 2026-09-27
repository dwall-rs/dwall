//! Network layer: HTTP client, download progress/task tracking, mirror resolution.

mod client;
mod mirror;
pub mod progress;
pub mod tasks;

pub use client::HttpClient;
pub use mirror::MirrorUrlResolver;
