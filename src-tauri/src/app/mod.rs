//! Core application module
//!
//! This module contains the core application logic including setup and command handlers.
//!
//! # Overview
//!
//! The app module serves as the entry point for Tauri commands and application setup.
//! It coordinates between the frontend and the backend services.
//!
//! # Modules
//!
//! - `commands`: Tauri command handlers that expose functionality to the frontend
//! - `setup`: Application initialization and setup logic
//! - `tracker`: Analytics tracking functionality

pub mod commands;
pub mod setup;
pub mod tracker;
