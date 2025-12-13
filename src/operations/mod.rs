//! File system operations for BA2 processing
//!
//! This module provides:
//! - Directory scanning for BA2 files
//! - BA2 extraction orchestration
//! - File validation
//! - Size parsing utilities
//! - Path handling utilities
//! - Retry logic for transient failures

pub mod extract;
pub mod path;
pub mod retry;
pub mod scan;

use crate::error::{Result, ValidationError};
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Cached regex for parsing size units (compiled once)
static SIZE_UNIT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"([KMGT]?B)").expect("Size regex pattern is valid"));

// Re-export scan module types and functions
pub use scan::{scan_for_ba2, ScanProgress};

// Re-export extract module types and functions
pub use extract::{
    extract_all, extract_ba2_file, ExtractionProgress, ExtractionResult, FileExtractionResult,
};

// Re-export path utilities
pub use path::{
    canonicalize_path, get_parent, is_valid_directory, is_valid_file, normalize_separators,
    paths_equal, resolve_path,
};

// Re-export retry utilities (Phase 2.8)
pub use retry::{retry, retry_with_config, RetryConfig};

/// Information about a discovered BA2 file
#[derive(Debug, Clone)]
pub struct BA2FileInfo {
    /// File name (without path)
    pub file_name: String,

    /// File size in bytes
    pub file_size: u64,

    /// Number of files in the archive
    pub num_files: u32,

    /// Parent directory name
    pub dir_name: String,

    /// Full path to the file
    pub full_path: PathBuf,

    /// Whether the file appears to be corrupted
    pub is_bad: bool,
}

/// Parse a size string (e.g., "10MB", "1.5GB") into bytes
///
/// This function matches the Python implementation's behavior:
/// - Uses base-1000 units (1KB = 1000 bytes, not 1024)
/// - Case-insensitive
/// - Supports units: B, KB, MB, GB, TB
/// - Handles floating point numbers
///
/// # Examples
///
/// ```
/// use unpackrr::operations::parse_size;
///
/// assert_eq!(parse_size("100B").unwrap(), 100);
/// assert_eq!(parse_size("1KB").unwrap(), 1000);
/// assert_eq!(parse_size("1.5MB").unwrap(), 1_500_000);
/// assert_eq!(parse_size("10GB").unwrap(), 10_000_000_000);
/// ```
///
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
pub fn parse_size(size_str: &str) -> Result<u64> {
    let mut size_str = size_str.trim().to_uppercase();

    // Add 'B' suffix if not present
    if !size_str.ends_with('B') {
        size_str.push('B');
    }

    // Use cached regex to separate number from unit (mimicking Python's re.sub)
    // Pattern: r"([KMGT]?B)" -> r" \1"
    // This inserts a space before the unit if not already there
    let size_str = SIZE_UNIT_REGEX.replace(&size_str, " $1");

    // Split into parts and parse
    let parts: Vec<&str> = size_str.split_whitespace().collect();

    if parts.len() != 2 {
        return Err(ValidationError::InvalidSize(size_str.to_string()).into());
    }

    let number_str = parts[0];
    let unit_str = parts[1];

    let number: f64 = number_str
        .parse()
        .map_err(|_| ValidationError::InvalidSize(size_str.to_string()))?;

    // Python uses base-1000 units, not base-1024
    let multiplier: u64 = match unit_str {
        "B" => 1,
        "KB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        _ => return Err(ValidationError::InvalidSize(size_str.to_string()).into()),
    };

    Ok((number * multiplier as f64) as u64)
}

/// Format a size in bytes to human-readable format
pub fn format_size(bytes: u64) -> String {
    humansize::format_size(bytes, humansize::BINARY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("100B").unwrap(), 100);
        assert_eq!(parse_size("1KB").unwrap(), 1_000);
        assert_eq!(parse_size("1MB").unwrap(), 1_000_000);
        assert_eq!(parse_size("1.5MB").unwrap(), 1_500_000);
        assert_eq!(parse_size("10GB").unwrap(), 10_000_000_000);
    }

    #[test]
    fn test_parse_size_case_insensitive() {
        assert_eq!(parse_size("1mb").unwrap(), parse_size("1MB").unwrap());
        assert_eq!(parse_size("1kb").unwrap(), 1_000);
    }

    #[test]
    fn test_parse_size_no_suffix() {
        // Should add 'B' suffix if not present
        assert_eq!(parse_size("100").unwrap(), 100);
        assert_eq!(parse_size("1K").unwrap(), 1_000);
    }

    #[test]
    fn test_parse_size_with_spaces() {
        assert_eq!(parse_size("1 MB").unwrap(), 1_000_000);
        assert_eq!(parse_size(" 100 KB ").unwrap(), 100_000);
    }

    #[test]
    fn test_parse_size_invalid() {
        assert!(parse_size("invalid").is_err());
        assert!(parse_size("").is_err());
        assert!(parse_size("MB").is_err());
    }

    #[test]
    fn test_format_size() {
        let formatted = format_size(1024);
        assert!(formatted.contains("1"));
        assert!(formatted.contains("Ki")); // humansize uses Ki for binary
    }
}
