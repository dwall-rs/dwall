//! Thumbnail management functionality
//!
//! This module provides functionality for caching and managing thumbnail images.

pub(crate) mod cleanup;
pub mod error;
pub(crate) mod storage;
pub(crate) mod store;
pub(crate) mod thumbnail_cache;

// Re-export the public API
pub use self::thumbnail_cache::{
    ThumbnailCache, clear_thumbnail_cache, get_or_save_cached_thumbnails,
};
pub(crate) use error::ThumbnailError;
