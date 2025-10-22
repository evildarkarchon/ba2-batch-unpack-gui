//! BA2 file format support and BSArch.exe integration
//!
//! This module provides:
//! - BA2 header parsing and validation
//! - File count extraction without full extraction
//! - Integration with BSArch.exe for extraction
//!
//! Note: We use BSArch.exe (MPL-2.0 licensed) as the extraction engine.
//! This module wraps it with a Rust-friendly API.

use crate::error::{BA2Error, Result};
use std::path::{Path, PathBuf};

/// BA2 archive header
#[derive(Debug, Clone)]
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

    /// Parse BA2 header from a file
    pub fn parse(_path: &Path) -> Result<Self> {
        // TODO: Implement in Phase 1.4
        Err(BA2Error::ExtractionFailed {
            path: _path.to_path_buf(),
            reason: "Not yet implemented".to_string(),
        }
        .into())
    }

    /// Validate the header
    pub fn validate(&self) -> Result<()> {
        if &self.magic != Self::MAGIC {
            return Err(BA2Error::InvalidMagic {
                path: PathBuf::from("unknown"),
            }
            .into());
        }
        Ok(())
    }
}

/// Get the number of files in a BA2 archive without extracting
pub fn num_files_in_ba2(_path: &Path) -> Result<u32> {
    // TODO: Implement in Phase 1.4
    Ok(0)
}

/// Check if a file is a valid BA2 archive
pub fn is_valid_ba2(_path: &Path) -> bool {
    // TODO: Implement in Phase 1.4
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ba2_magic() {
        assert_eq!(BA2Header::MAGIC, b"BTDX");
    }
}
