//! Unpackrr-rs: High-performance BA2 batch unpacker with Fluent Design UI
//!
//! This is a Rust port of the Python-based BA2 batch unpacker GUI,
//! leveraging Rust's performance, safety, and modern tooling.
//!
//! # Architecture
//!
//! - `error`: Custom error types and error handling
//! - `config`: Configuration management and persistence
//! - `ba2`: BA2 file format support and BSArch.exe integration
//! - `operations`: File system operations (scanning, extraction, validation)
//! - `models`: Data models for UI display
//! - `ui`: Slint UI components and integration
//! - `logging`: Logging configuration and file rotation
//! - `log_viewer`: Log viewer for displaying and filtering application logs
//! - `update_checker`: GitHub release update checking
//! - `platform`: Platform-specific functionality (Windows registry, etc.)

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc)]

pub mod ba2;
pub mod config;
pub mod error;
pub mod log_viewer;
pub mod logging;
pub mod models;
pub mod operations;
pub mod platform;
pub mod ui;
pub mod update_checker;

pub use error::{Error, Result};
