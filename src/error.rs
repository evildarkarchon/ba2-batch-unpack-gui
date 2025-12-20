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
        /// Path to the configuration file
        path: PathBuf,
        /// Underlying I/O error
        source: std::io::Error,
    },

    /// Failed to save configuration file
    #[error("Failed to save configuration to {path}: {source}")]
    SaveFailed {
        /// Path to the configuration file
        path: PathBuf,
        /// Underlying I/O error
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
        /// The invalid regex pattern
        pattern: String,
        /// Underlying regex compilation error
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
    InvalidMagic {
        /// Path to the BA2 file with invalid magic number
        path: PathBuf,
    },

    /// Unsupported BA2 version
    #[error("Unsupported BA2 version {version} in file {path}")]
    UnsupportedVersion {
        /// The unsupported version number
        version: u32,
        /// Path to the BA2 file
        path: PathBuf,
    },

    /// Corrupted BA2 file
    #[error("Corrupted BA2 file: {path} - {reason}")]
    Corrupted {
        /// Path to the corrupted BA2 file
        path: PathBuf,
        /// Reason for corruption
        reason: String,
    },

    /// Failed to extract BA2 file
    #[error("Failed to extract {path}: {reason}")]
    ExtractionFailed {
        /// Path to the BA2 file
        path: PathBuf,
        /// Reason for extraction failure
        reason: String,
    },

    /// BSArch.exe not found
    #[error("BSArch.exe not found at expected location: {path}")]
    BSArchNotFound {
        /// Expected path to BSArch.exe
        path: PathBuf,
    },

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
    pub const fn is_ba2_corrupted(&self) -> bool {
        matches!(self, Self::BA2(BA2Error::Corrupted { .. }))
    }

    /// Check if this error might be transient (retryable)
    ///
    /// Transient errors include temporary I/O issues, network problems,
    /// or file access conflicts that might resolve with a retry.
    #[must_use]
    pub fn is_transient(&self) -> bool {
        match self {
            Self::IO(e) => {
                use std::io::ErrorKind;
                matches!(
                    e.kind(),
                    ErrorKind::Interrupted
                        | ErrorKind::WouldBlock
                        | ErrorKind::TimedOut
                        | ErrorKind::PermissionDenied // Might be temporary file lock
                        | ErrorKind::AddrInUse
                )
            }
            Self::BA2(BA2Error::BSArchExecFailed(_)) => true, // External tool might be busy
            _ => false,
        }
    }

    /// Get a user-friendly error message (Phase 2.8)
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            Self::Config(e) => match e {
                ConfigError::LoadFailed { path, .. } => {
                    format!("Failed to load settings from '{}'", path.display())
                }
                ConfigError::SaveFailed { path, .. } => {
                    format!("Failed to save settings to '{}'", path.display())
                }
                ConfigError::InvalidFormat(msg) => format!("Invalid settings format: {msg}"),
                ConfigError::ValidationFailed(msg) => {
                    format!("Settings validation failed: {msg}")
                }
                ConfigError::InvalidRegex { pattern, .. } => {
                    format!("Invalid pattern: '{pattern}'")
                }
                ConfigError::InvalidPath(path) => {
                    format!("Invalid path in settings: '{}'", path.display())
                }
            },
            Self::BA2(e) => match e {
                BA2Error::InvalidMagic { path } => {
                    format!("'{}' is not a valid BA2 file", path.display())
                }
                BA2Error::UnsupportedVersion { version, path } => {
                    format!(
                        "'{}' uses unsupported BA2 version {}",
                        path.display(),
                        version
                    )
                }
                BA2Error::Corrupted { path, reason } => {
                    format!("BA2 file '{}' is corrupted: {}", path.display(), reason)
                }
                BA2Error::ExtractionFailed { path, reason } => {
                    format!("Failed to extract '{}': {}", path.display(), reason)
                }
                BA2Error::BSArchNotFound { path } => {
                    format!("BA2 extraction tool not found at '{}'", path.display())
                }
                BA2Error::BSArchExecFailed(msg) => {
                    format!("BA2 extraction tool failed: {msg}")
                }
            },
            Self::IO(e) => {
                use std::io::ErrorKind;
                match e.kind() {
                    ErrorKind::NotFound => "File or folder not found".to_string(),
                    ErrorKind::PermissionDenied => {
                        "Permission denied - check file permissions".to_string()
                    }
                    ErrorKind::AlreadyExists => "File already exists".to_string(),
                    ErrorKind::InvalidInput => "Invalid input provided".to_string(),
                    ErrorKind::TimedOut => "Operation timed out".to_string(),
                    ErrorKind::Interrupted => "Operation was interrupted".to_string(),
                    _ => format!("File operation failed: {e}"),
                }
            }
            Self::Validation(e) => match e {
                ValidationError::InvalidInput(msg) => format!("Invalid input: {msg}"),
                ValidationError::PathNotFound(path) => {
                    format!("Path not found: '{}'", path.display())
                }
                ValidationError::NotADirectory(path) => {
                    format!("'{}' is not a folder", path.display())
                }
                ValidationError::NotAFile(path) => {
                    format!("'{}' is not a file", path.display())
                }
                ValidationError::InvalidSize(msg) => {
                    format!("Invalid size format: {msg}")
                }
            },
            Self::Other(msg) => msg.clone(),
        }
    }

    /// Get recovery suggestions for the error (Phase 2.8)
    ///
    /// Returns a list of actionable suggestions the user can try
    /// to resolve or work around the error.
    #[must_use]
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            Self::Config(ConfigError::LoadFailed { .. }) => vec![
                "Check that the settings file exists and is readable".to_string(),
                "Try deleting the settings file to restore defaults".to_string(),
                "Ensure you have read permissions for the file".to_string(),
            ],
            Self::Config(ConfigError::SaveFailed { .. }) => vec![
                "Check that you have write permissions".to_string(),
                "Ensure there is enough disk space".to_string(),
                "Try running the application as administrator".to_string(),
            ],
            Self::Config(ConfigError::InvalidRegex { .. }) => vec![
                "Check your file filter patterns in Settings".to_string(),
                "Use simpler patterns or wildcards instead of regex".to_string(),
                "Reset to default patterns".to_string(),
            ],
            Self::BA2(BA2Error::Corrupted { .. }) => vec![
                "Try re-downloading the mod from its source".to_string(),
                "Verify the file integrity if available".to_string(),
                "Skip this file and continue with others".to_string(),
            ],
            Self::BA2(BA2Error::BSArchNotFound { .. }) => vec![
                "Specify the BA2 extraction tool path in Settings > Advanced".to_string(),
                "Download BSArch.exe from TES5Edit project".to_string(),
                "Check if an antivirus blocked the file".to_string(),
            ],
            Self::BA2(BA2Error::BSArchExecFailed(_)) => vec![
                "Try running the extraction again".to_string(),
                "Check if another program is using the files".to_string(),
                "Ensure the extraction tool has execute permissions".to_string(),
            ],
            Self::IO(e) if e.kind() == std::io::ErrorKind::PermissionDenied => vec![
                "Close any programs that might be using these files".to_string(),
                "Run the application as administrator".to_string(),
                "Check file and folder permissions".to_string(),
            ],
            Self::IO(e) if e.kind() == std::io::ErrorKind::NotFound => vec![
                "Verify the path exists and hasn't been moved".to_string(),
                "Check for typos in the file path".to_string(),
                "Ensure the drive is connected and accessible".to_string(),
            ],
            Self::Validation(ValidationError::PathNotFound(_)) => vec![
                "Browse for the folder instead of typing the path".to_string(),
                "Verify the folder hasn't been moved or deleted".to_string(),
                "Check that network drives are connected".to_string(),
            ],
            Self::Validation(ValidationError::InvalidSize(_)) => vec![
                "Use format like '500MB' or '2GB'".to_string(),
                "Valid units: B, KB, MB, GB, TB".to_string(),
                "Numbers without units are treated as bytes".to_string(),
            ],
            _ => vec!["Try the operation again".to_string()],
        }
    }

    /// Get detailed technical error information (Phase 2.8)
    ///
    /// Returns a detailed error report including the full error chain,
    /// suitable for debugging or copying to bug reports.
    #[must_use]
    pub fn detailed_report(&self) -> String {
        use std::fmt::Write;
        let mut report = String::new();

        // Error type
        report.push_str("Error Type: ");
        report.push_str(match self {
            Self::Config(_) => "Configuration",
            Self::BA2(_) => "BA2 File Format",
            Self::IO(_) => "File System I/O",
            Self::Validation(_) => "Input Validation",
            Self::Other(_) => "General",
        });
        report.push_str("\n\n");

        // Main error message
        report.push_str("Error: ");
        report.push_str(&self.to_string());
        report.push_str("\n\n");

        // User-friendly message
        report.push_str("User Message: ");
        report.push_str(&self.user_message());
        report.push_str("\n\n");

        // Transient flag
        if self.is_transient() {
            report.push_str("Note: This error might be temporary. Retrying may succeed.\n\n");
        }

        // Recovery suggestions
        let suggestions = self.recovery_suggestions();
        if !suggestions.is_empty() {
            report.push_str("Recovery Suggestions:\n");
            for (i, suggestion) in suggestions.iter().enumerate() {
                let _ = writeln!(report, "{}. {}", i + 1, suggestion);
            }
            report.push('\n');
        }

        // Version info for bug reports
        let _ = writeln!(report, "Version: {}", env!("CARGO_PKG_VERSION"));
        let _ = writeln!(report, "Platform: {}", std::env::consts::OS);

        report
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
        assert!(msg.contains("Path not found"));
    }

    #[test]
    fn test_is_transient_io_interrupted() {
        let err = Error::IO(std::io::Error::from(std::io::ErrorKind::Interrupted));
        assert!(err.is_transient());
    }

    #[test]
    fn test_is_transient_permanent_error() {
        let err = Error::Validation(ValidationError::PathNotFound(PathBuf::from("/test")));
        assert!(!err.is_transient());
    }

    #[test]
    fn test_recovery_suggestions() {
        let err = Error::BA2(BA2Error::Corrupted {
            path: PathBuf::from("test.ba2"),
            reason: "invalid header".to_string(),
        });
        let suggestions = err.recovery_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("re-downloading")));
    }

    #[test]
    fn test_detailed_report() {
        let err = Error::other("test error");
        let report = err.detailed_report();
        assert!(report.contains("Error Type"));
        assert!(report.contains("User Message"));
        assert!(report.contains("Version"));
        assert!(report.contains("Platform"));
    }

    #[test]
    fn test_user_message_io_not_found() {
        let err = Error::IO(std::io::Error::from(std::io::ErrorKind::NotFound));
        let msg = err.user_message();
        assert_eq!(msg, "File or folder not found");
    }
}
