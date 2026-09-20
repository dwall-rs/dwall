//! Thumbnail cache management.

pub(crate) mod cache;
pub(crate) mod cleanup;
pub mod error;
pub(crate) mod storage;
pub(crate) mod store;

pub use self::cache::{ThumbnailCache, clear_thumbnail_cache, get_or_save_cached_thumbnails};
pub(crate) use error::ThumbnailError;
