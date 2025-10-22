//! BA2 file format support and BSArch.exe integration
//!
//! This module provides:
//! - BA2 header parsing and validation
//! - File count extraction without full extraction
//! - Integration with BSArch.exe for extraction
//!
//! Note: We use BSArch.exe (MPL-2.0 licensed) as the extraction engine.
//! This module wraps it with a Rust-friendly API.

mod extractor;

pub use extractor::{extract_ba2, list_ba2, BSArchConfig};

use crate::error::{BA2Error, Result};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// BA2 archive header
///
/// The BA2 format header consists of:
/// - Magic number: "BTDX" (4 bytes)
/// - Version: u32 (4 bytes)
/// - Archive type: 4-character string (4 bytes) - "GNRL", "DX10", etc.
/// - File count: u32 (4 bytes)
/// - Names offset: u64 (8 bytes)
///
/// Total: 24 bytes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BA2Header {
    /// Magic number - should be "BTDX"
    pub magic: [u8; 4],

    /// BA2 format version
    pub version: u32,

    /// Archive type (GNRL, DX10, etc.)
    pub archive_type: String,

    /// Number of files in the archive
    pub file_count: u32,

    /// Offset to file names table
    pub names_offset: u64,
}

impl BA2Header {
    /// Expected magic number for BA2 files
    pub const MAGIC: &'static [u8; 4] = b"BTDX";

    /// Header size in bytes
    pub const HEADER_SIZE: usize = 24;

    /// Parse BA2 header from a file
    pub fn parse(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|e| BA2Error::ExtractionFailed {
            path: path.to_path_buf(),
            reason: format!("Failed to open file: {e}"),
        })?;

        let mut reader = BufReader::new(file);
        Self::parse_from_reader(&mut reader, path)
    }

    /// Parse BA2 header from a reader
    pub fn parse_from_reader<R: Read>(reader: &mut R, path: &Path) -> Result<Self> {
        let mut buffer = [0u8; Self::HEADER_SIZE];

        reader.read_exact(&mut buffer).map_err(|e| BA2Error::Corrupted {
            path: path.to_path_buf(),
            reason: format!("Failed to read header: {e}"),
        })?;

        // Parse magic number
        let magic = [buffer[0], buffer[1], buffer[2], buffer[3]];

        // Parse version (little-endian u32)
        let version = u32::from_le_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);

        // Parse archive type (4-byte string)
        let archive_type = String::from_utf8_lossy(&buffer[8..12])
            .trim_end_matches('\0')
            .to_string();

        // Parse file count (little-endian u32)
        let file_count = u32::from_le_bytes([buffer[12], buffer[13], buffer[14], buffer[15]]);

        // Parse names offset (little-endian u64)
        let names_offset = u64::from_le_bytes([
            buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22],
            buffer[23],
        ]);

        let header = Self {
            magic,
            version,
            archive_type,
            file_count,
            names_offset,
        };

        // Validate the header
        header.validate(path)?;

        Ok(header)
    }

    /// Validate the header
    pub fn validate(&self, path: &Path) -> Result<()> {
        if &self.magic != Self::MAGIC {
            return Err(BA2Error::InvalidMagic {
                path: path.to_path_buf(),
            }
            .into());
        }

        // Validate known archive types
        match self.archive_type.as_str() {
            "GNRL" | "DX10" => Ok(()),
            _ => {
                tracing::warn!(
                    "Unknown BA2 archive type '{}' in file: {}",
                    self.archive_type,
                    path.display()
                );
                Ok(())
            }
        }
    }

    /// Check if this is a General archive
    pub fn is_general(&self) -> bool {
        self.archive_type == "GNRL"
    }

    /// Check if this is a Texture archive
    pub fn is_texture(&self) -> bool {
        self.archive_type == "DX10"
    }
}

/// Get the number of files in a BA2 archive without extracting
///
/// This function only reads the header (24 bytes) to extract the file count.
/// It's much faster than extracting the entire archive.
pub fn num_files_in_ba2(path: &Path) -> Result<u32> {
    let header = BA2Header::parse(path)?;
    Ok(header.file_count)
}

/// Check if a file is a valid BA2 archive
///
/// This performs a quick validation by:
/// 1. Checking if the file exists
/// 2. Checking if it has the correct magic number
/// 3. Attempting to parse the header
pub fn is_valid_ba2(path: &Path) -> bool {
    if !path.exists() || !path.is_file() {
        return false;
    }

    match BA2Header::parse(path) {
        Ok(_) => true,
        Err(e) => {
            tracing::debug!("BA2 validation failed for {}: {}", path.display(), e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_ba2_magic() {
        assert_eq!(BA2Header::MAGIC, b"BTDX");
    }

    #[test]
    fn test_header_size() {
        assert_eq!(BA2Header::HEADER_SIZE, 24);
    }

    #[test]
    fn test_parse_valid_header() {
        // Create a valid BA2 header
        let mut data = Vec::new();
        data.extend_from_slice(b"BTDX"); // Magic
        data.extend_from_slice(&1u32.to_le_bytes()); // Version
        data.extend_from_slice(b"GNRL"); // Archive type
        data.extend_from_slice(&100u32.to_le_bytes()); // File count
        data.extend_from_slice(&1024u64.to_le_bytes()); // Names offset

        let mut cursor = Cursor::new(data);
        let path = PathBuf::from("test.ba2");
        let header = BA2Header::parse_from_reader(&mut cursor, &path).unwrap();

        assert_eq!(header.magic, *b"BTDX");
        assert_eq!(header.version, 1);
        assert_eq!(header.archive_type, "GNRL");
        assert_eq!(header.file_count, 100);
        assert_eq!(header.names_offset, 1024);
    }

    #[test]
    fn test_parse_invalid_magic() {
        // Create header with invalid magic
        let mut data = Vec::new();
        data.extend_from_slice(b"XXXX"); // Invalid magic
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(b"GNRL");
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(&1024u64.to_le_bytes());

        let mut cursor = Cursor::new(data);
        let path = PathBuf::from("test.ba2");
        let result = BA2Header::parse_from_reader(&mut cursor, &path);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::Error::BA2(BA2Error::InvalidMagic { .. })
        ));
    }

    #[test]
    fn test_is_general() {
        let header = BA2Header {
            magic: *b"BTDX",
            version: 1,
            archive_type: "GNRL".to_string(),
            file_count: 100,
            names_offset: 1024,
        };
        assert!(header.is_general());
        assert!(!header.is_texture());
    }

    #[test]
    fn test_is_texture() {
        let header = BA2Header {
            magic: *b"BTDX",
            version: 1,
            archive_type: "DX10".to_string(),
            file_count: 100,
            names_offset: 1024,
        };
        assert!(header.is_texture());
        assert!(!header.is_general());
    }

    #[test]
    fn test_parse_truncated_header() {
        // Create truncated data (less than 24 bytes)
        let data = vec![0u8; 10];
        let mut cursor = Cursor::new(data);
        let path = PathBuf::from("test.ba2");
        let result = BA2Header::parse_from_reader(&mut cursor, &path);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::error::Error::BA2(BA2Error::Corrupted { .. })
        ));
    }
}
