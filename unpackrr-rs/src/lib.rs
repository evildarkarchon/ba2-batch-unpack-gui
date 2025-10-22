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
//! - `ui`: Slint UI components and integration

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::must_use_candidate, clippy::missing_errors_doc)]

pub mod ba2;
pub mod config;
pub mod error;
pub mod operations;
pub mod ui;

pub use error::{Error, Result};
