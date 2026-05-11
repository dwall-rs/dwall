//! Services module
//!
//! This module contains higher-level services that coordinate between domain logic and infrastructure.
//!
//! # Overview
//!
//! Services implement application-specific use cases by coordinating between the domain
//! layer (business logic) and the infrastructure layer (external systems).
//!
//! # Modules
//!
//! - `theme_service`: Theme application and management functionality
//! - `cache`: Thumbnail caching and management
//! - `download_service`: Theme download coordination

pub mod cache;
pub mod download_service;
pub mod theme_service;
