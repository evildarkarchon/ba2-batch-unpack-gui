//! BSArch.exe integration for BA2 extraction
//!
//! This module provides a Rust wrapper around BSArch.exe (MPL-2.0 licensed)
//! for extracting and listing BA2 archives.
//!
//! BSArch.exe is the industry-standard tool for BA2 archives and is used
//! by many Bethesda modding tools. We wrap it rather than reimplement it.

use crate::error::{BA2Error, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Configuration for BSArch.exe operations
#[derive(Debug, Clone)]
pub struct BSArchConfig {
    /// Path to BSArch.exe
    pub bsarch_path: PathBuf,

    /// Custom extraction path (None = extract to source directory)
    pub extraction_path: Option<PathBuf>,

    /// Use a temporary directory for extraction
    pub use_temp: bool,
}

impl BSArchConfig {
    /// Create a new BSArch configuration
    pub fn new(bsarch_path: PathBuf) -> Self {
        Self {
            bsarch_path,
            extraction_path: None,
            use_temp: false,
        }
    }

    /// Set custom extraction path
    pub fn with_extraction_path(mut self, path: PathBuf) -> Self {
        self.extraction_path = Some(path);
        self
    }

    /// Set whether to use temporary directory
    pub fn with_temp(mut self, use_temp: bool) -> Self {
        self.use_temp = use_temp;
        self
    }

    /// Get the BSArch.exe path, checking if it exists
    pub fn validate(&self) -> Result<()> {
        if !self.bsarch_path.exists() {
            return Err(BA2Error::BSArchNotFound {
                path: self.bsarch_path.clone(),
            }
            .into());
        }
        Ok(())
    }
}

impl Default for BSArchConfig {
    fn default() -> Self {
        // Default to BSArch.exe in the current directory
        Self::new(PathBuf::from("BSArch.exe"))
    }
}

/// Extract a BA2 archive using BSArch.exe
///
/// # Arguments
/// * `ba2_path` - Path to the BA2 file to extract
/// * `config` - BSArch configuration
///
/// # Returns
/// * `Ok(())` on successful extraction
/// * `Err(BA2Error)` if extraction fails
pub fn extract_ba2(ba2_path: &Path, config: &BSArchConfig) -> Result<()> {
    config.validate()?;

    // Determine extraction path
    let extraction_path = if config.use_temp {
        // Create a temp directory in system temp
        let temp_dir = std::env::temp_dir().join(format!(
            "unpackrr_{}",
            ba2_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("temp")
        ));
        fs::create_dir_all(&temp_dir).map_err(|e| BA2Error::ExtractionFailed {
            path: ba2_path.to_path_buf(),
            reason: format!("Failed to create temp directory: {e}"),
        })?;
        temp_dir
    } else if let Some(custom_path) = &config.extraction_path {
        // Use custom extraction path
        if custom_path.is_absolute() {
            custom_path.clone()
        } else {
            // Relative to the BA2 file's directory
            ba2_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join(custom_path)
        }
    } else {
        // Extract to same directory as BA2 file
        ba2_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf()
    };

    // Create extraction directory if it doesn't exist
    if !extraction_path.exists() {
        fs::create_dir_all(&extraction_path).map_err(|e| BA2Error::ExtractionFailed {
            path: ba2_path.to_path_buf(),
            reason: format!("Failed to create extraction directory: {e}"),
        })?;
    }

    tracing::info!(
        "Extracting {} to {}",
        ba2_path.display(),
        extraction_path.display()
    );

    // Build command: BSArch.exe unpack <ba2_file> <output_dir>
    let mut cmd = Command::new(&config.bsarch_path);
    cmd.arg("unpack")
        .arg(ba2_path)
        .arg(&extraction_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // On Windows, hide the console window
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    // Execute the command
    let output = cmd.output().map_err(|e| BA2Error::BSArchExecFailed(format!("Failed to execute BSArch.exe: {e}")))?;

    // Check for errors in output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    tracing::debug!("BSArch stdout: {}", stdout);
    if !stderr.is_empty() {
        tracing::debug!("BSArch stderr: {}", stderr);
    }

    // BSArch.exe doesn't always return correct exit codes, so check for "Error:" in output
    if stdout.contains("Error:") || stdout.contains("error:") || stderr.contains("Error:") || stderr.contains("error:") {
        return Err(BA2Error::ExtractionFailed {
            path: ba2_path.to_path_buf(),
            reason: format!("BSArch.exe reported error: {stdout}"),
        }
        .into());
    }

    tracing::info!("Successfully extracted {}", ba2_path.display());

    // Clean up temp directory if used
    if config.use_temp {
        if let Err(e) = fs::remove_dir_all(&extraction_path) {
            tracing::warn!("Failed to clean up temp directory: {}", e);
        }
    }

    Ok(())
}

/// List contents of a BA2 archive using BSArch.exe
///
/// This is used to validate that a BA2 file is readable without extracting it.
///
/// # Arguments
/// * `ba2_path` - Path to the BA2 file to list
/// * `config` - BSArch configuration
///
/// # Returns
/// * `Ok(Vec<String>)` with list of files in the archive
/// * `Err(BA2Error)` if listing fails
pub fn list_ba2(ba2_path: &Path, config: &BSArchConfig) -> Result<Vec<String>> {
    config.validate()?;

    tracing::debug!("Listing contents of {}", ba2_path.display());

    // Build command: BSArch.exe <ba2_file> -list
    let mut cmd = Command::new(&config.bsarch_path);
    cmd.arg(ba2_path)
        .arg("-list")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // On Windows, hide the console window
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    // Execute the command
    let output = cmd.output().map_err(|e| BA2Error::BSArchExecFailed(format!("Failed to execute BSArch.exe: {e}")))?;

    // Check for errors in output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    tracing::debug!("BSArch stdout: {}", stdout);
    if !stderr.is_empty() {
        tracing::debug!("BSArch stderr: {}", stderr);
    }

    // BSArch.exe doesn't always return correct exit codes, so check for "Error:" in output
    if stdout.contains("Error:") || stdout.contains("error:") || stderr.contains("Error:") || stderr.contains("error:") {
        return Err(BA2Error::ExtractionFailed {
            path: ba2_path.to_path_buf(),
            reason: format!("BSArch.exe reported error: {stdout}"),
        }
        .into());
    }

    // Parse the output to extract file names
    let files: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !line.starts_with("Archive:"))
        .filter(|line| !line.starts_with("Files:"))
        .map(|line| line.trim().to_string())
        .collect();

    tracing::debug!("Found {} files in {}", files.len(), ba2_path.display());

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsarch_config_default() {
        let config = BSArchConfig::default();
        assert_eq!(config.bsarch_path, PathBuf::from("BSArch.exe"));
        assert!(config.extraction_path.is_none());
        assert!(!config.use_temp);
    }

    #[test]
    fn test_bsarch_config_with_extraction_path() {
        let config = BSArchConfig::default().with_extraction_path(PathBuf::from("/custom/path"));
        assert_eq!(
            config.extraction_path,
            Some(PathBuf::from("/custom/path"))
        );
    }

    #[test]
    fn test_bsarch_config_with_temp() {
        let config = BSArchConfig::default().with_temp(true);
        assert!(config.use_temp);
    }

    #[test]
    fn test_bsarch_config_validation_fails() {
        let config = BSArchConfig::new(PathBuf::from("/nonexistent/BSArch.exe"));
        assert!(config.validate().is_err());
    }
}
