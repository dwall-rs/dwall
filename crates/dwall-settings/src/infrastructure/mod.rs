//! Infrastructure module
//!
//! This module contains adapters for external systems and low-level operations.
//!
//! # Overview
//!
//! The infrastructure layer provides implementations for interacting with external
//! systems like the file system, network, Windows registry, and process management.
//!
//! # Modules
//!
//! - `filesystem`: File system operations and directory management
//! - `process`: Process creation, management, and termination
//! - `registry`: Windows registry operations for auto-start and other settings
//! - `network`: Network operations including HTTP requests and downloads
//! - `window`: Window management and UI-related operations

pub mod filesystem;
pub mod network;
pub mod process;
pub mod registry;
pub mod window;
