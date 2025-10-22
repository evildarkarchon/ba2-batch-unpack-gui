//! File system operations for BA2 processing
//!
//! This module provides:
//! - Directory scanning for BA2 files
//! - BA2 extraction orchestration
//! - File validation
//! - Size parsing utilities

use crate::error::{Result, ValidationError};
use std::path::{Path, PathBuf};

/// Scan a directory for BA2 files
///
/// Searches second-tier folders for BA2 files matching configured patterns
pub async fn scan_for_ba2(_path: &Path) -> Result<Vec<BA2FileInfo>> {
    // TODO: Implement in Phase 1.5
    Ok(Vec::new())
}

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
pub fn parse_size(size_str: &str) -> Result<u64> {
    let size_str = size_str.trim().to_uppercase();

    // Extract number and unit
    let mut num_str = String::new();
    let mut unit_str = String::new();

    for ch in size_str.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num_str.push(ch);
        } else if ch.is_alphabetic() {
            unit_str.push(ch);
        }
    }

    let num: f64 = num_str
        .parse()
        .map_err(|_| ValidationError::InvalidSize(size_str.clone()))?;

    let multiplier: u64 = match unit_str.as_str() {
        "B" | "" => 1,
        "KB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        "TB" => 1024 * 1024 * 1024 * 1024,
        _ => return Err(ValidationError::InvalidSize(size_str).into()),
    };

    Ok((num * multiplier as f64) as u64)
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
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1.5MB").unwrap(), (1.5 * 1024.0 * 1024.0) as u64);
    }

    #[test]
    fn test_parse_size_case_insensitive() {
        assert_eq!(parse_size("1mb").unwrap(), parse_size("1MB").unwrap());
    }

    #[test]
    fn test_format_size() {
        let formatted = format_size(1024);
        assert!(formatted.contains("1"));
        assert!(formatted.contains("Ki")); // humansize uses Ki for binary
    }
}
