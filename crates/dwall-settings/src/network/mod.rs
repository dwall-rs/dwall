//! Network layer: HTTP client, download progress/task tracking, mirror resolution.

mod client;
mod mirror;
pub mod progress;
pub mod tasks;
#[cfg(test)]
pub(crate) mod testing;

pub use client::HttpClient;
pub use mirror::MirrorUrlResolver;
