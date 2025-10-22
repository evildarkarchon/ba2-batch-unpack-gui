//! Error types for Unpackrr-rs
//!
//! This module defines comprehensive error types for all operations in the application.
//! We use `thiserror` for library-level errors and `anyhow` for application-level error handling.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using our custom Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for Unpackrr-rs operations
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    /// BA2 file format or parsing errors
    #[error("BA2 error: {0}")]
    BA2(#[from] BA2Error),

    /// File system I/O errors
    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    /// Input validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

/// Configuration-related errors
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to load configuration file
    #[error("Failed to load configuration from {path}: {source}")]
    LoadFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to save configuration file
    #[error("Failed to save configuration to {path}: {source}")]
    SaveFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),

    /// Configuration validation failed
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),

    /// Invalid regex pattern in configuration
    #[error("Invalid regex pattern '{pattern}': {source}")]
    InvalidRegex {
        pattern: String,
        source: regex::Error,
    },

    /// Invalid path in configuration
    #[error("Invalid path in configuration: {0}")]
    InvalidPath(PathBuf),
}

/// BA2 file format and parsing errors
#[derive(Error, Debug)]
pub enum BA2Error {
    /// Invalid BA2 magic number
    #[error("Invalid BA2 magic number in file {path}")]
    InvalidMagic { path: PathBuf },

    /// Unsupported BA2 version
    #[error("Unsupported BA2 version {version} in file {path}")]
    UnsupportedVersion { version: u32, path: PathBuf },

    /// Corrupted BA2 file
    #[error("Corrupted BA2 file: {path} - {reason}")]
    Corrupted { path: PathBuf, reason: String },

    /// Failed to extract BA2 file
    #[error("Failed to extract {path}: {reason}")]
    ExtractionFailed { path: PathBuf, reason: String },

    /// BSArch.exe not found
    #[error("BSArch.exe not found at expected location: {path}")]
    BSArchNotFound { path: PathBuf },

    /// BSArch.exe execution failed
    #[error("BSArch.exe execution failed: {0}")]
    BSArchExecFailed(String),
}

/// Input validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Empty or invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// File or directory does not exist
    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),

    /// Path is not a directory
    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),

    /// Path is not a file
    #[error("Path is not a file: {0}")]
    NotAFile(PathBuf),

    /// Invalid size format
    #[error("Invalid size format: {0}")]
    InvalidSize(String),
}

impl Error {
    /// Create a generic error with a message
    #[must_use]
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }

    /// Check if this error is a BA2 corruption error
    #[must_use]
    pub fn is_ba2_corrupted(&self) -> bool {
        matches!(self, Self::BA2(BA2Error::Corrupted { .. }))
    }

    /// Get a user-friendly error message
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Config(e) => format!("Configuration error: {e}"),
            Self::BA2(BA2Error::Corrupted { path, reason }) => {
                format!("The BA2 file '{}' appears to be corrupted: {}", path.display(), reason)
            }
            Self::BA2(e) => format!("BA2 file error: {e}"),
            Self::IO(e) => format!("File operation failed: {e}"),
            Self::Validation(e) => format!("Validation error: {e}"),
            Self::Other(msg) => msg.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::other("test error");
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_ba2_corrupted_check() {
        let err = Error::BA2(BA2Error::Corrupted {
            path: PathBuf::from("test.ba2"),
            reason: "invalid header".to_string(),
        });
        assert!(err.is_ba2_corrupted());
    }

    #[test]
    fn test_user_message() {
        let err = Error::Validation(ValidationError::PathNotFound(PathBuf::from("/test/path")));
        let msg = err.user_message();
        assert!(msg.contains("Validation error"));
    }
}
