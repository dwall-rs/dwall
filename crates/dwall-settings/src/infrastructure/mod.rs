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
//! - `filesystem`: File system operations (pure technical)
//! - `process`: Process management (pure technical)
//! - `registry`: Windows registry operations
//! - `network`: Network operations (pure technical)
//! - `window`: Window management (pure technical)
//! - `logging`: Log parsing utilities

pub mod filesystem;
pub mod logging;
pub mod network;
pub mod process;
pub mod registry;
pub mod window;
