//! BA2 file extraction operations
//!
//! This module handles the orchestration of BA2 file extraction using BSArch.exe.
//! It provides progress tracking, error handling, and batch extraction capabilities.

use crate::config::AppConfig;
use crate::error::{BA2Error, Result};
use crate::models::FileEntry;
use futures::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{mpsc, Semaphore};

/// Progress updates during extraction
#[derive(Debug, Clone)]
pub enum ExtractionProgress {
    /// Started extraction of a file
    Started {
        /// File being extracted
        file_name: String,
        /// Current file number (1-indexed)
        current: usize,
        /// Total number of files to extract
        total: usize,
    },

    /// File extraction completed
    Completed {
        /// File that was extracted
        file_name: String,
        /// Whether extraction was successful
        success: bool,
        /// Error message if extraction failed
        error: Option<String>,
    },

    /// All extractions finished
    Finished {
        /// Number of successful extractions
        successful: usize,
        /// Number of failed extractions
        failed: usize,
    },
}

/// Result of a single file extraction
#[derive(Debug, Clone)]
pub struct FileExtractionResult {
    /// Path to the BA2 file
    pub file_path: PathBuf,
    /// Whether extraction was successful
    pub success: bool,
    /// Error message if extraction failed
    pub error: Option<String>,
}

/// Result of batch extraction
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Individual file results
    pub file_results: Vec<FileExtractionResult>,
    /// Number of successful extractions
    pub successful: usize,
    /// Number of failed extractions
    pub failed: usize,
}

impl ExtractionResult {
    /// Create a new empty result
    pub const fn new() -> Self {
        Self {
            file_results: Vec::new(),
            successful: 0,
            failed: 0,
        }
    }

    /// Add a file result
    pub fn add_result(&mut self, result: FileExtractionResult) {
        if result.success {
            self.successful += 1;
        } else {
            self.failed += 1;
        }
        self.file_results.push(result);
    }

    /// Get list of successfully extracted files
    pub fn successful_files(&self) -> Vec<&PathBuf> {
        self.file_results
            .iter()
            .filter(|r| r.success)
            .map(|r| &r.file_path)
            .collect()
    }

    /// Get list of failed files
    pub fn failed_files(&self) -> Vec<&PathBuf> {
        self.file_results
            .iter()
            .filter(|r| !r.success)
            .map(|r| &r.file_path)
            .collect()
    }
}

impl Default for ExtractionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract a single BA2 file using BSArch.exe
///
/// # Arguments
///
/// * `ba2_path` - Path to the BA2 file to extract
/// * `output_dir` - Directory to extract files to (defaults to BA2's parent directory)
/// * `bsarch_path` - Path to BSArch.exe
///
/// # Returns
///
/// `Ok(())` if extraction succeeds, `Err` otherwise
///
pub async fn extract_ba2_file(
    ba2_path: &Path,
    output_dir: Option<&Path>,
    bsarch_path: &Path,
) -> Result<()> {
    // Validate BA2 file exists
    if !ba2_path.exists() {
        return Err(BA2Error::ExtractionFailed {
            path: ba2_path.to_path_buf(),
            reason: "File not found".to_string(),
        }
        .into());
    }

    // Validate BSArch.exe exists
    if !bsarch_path.exists() {
        return Err(BA2Error::BSArchNotFound {
            path: bsarch_path.to_path_buf(),
        }
        .into());
    }

    // Determine output directory
    let Some(output_path) = output_dir.or_else(|| ba2_path.parent()) else {
        return Err(BA2Error::ExtractionFailed {
            path: ba2_path.to_path_buf(),
            reason: "BA2 file path has no parent directory".to_string(),
        }
        .into());
    };

    // Build BSArch command
    // Format: BSArch.exe unpack <ba2_file> <output_dir>
    let mut cmd = Command::new(bsarch_path);
    cmd.arg("unpack")
        .arg(ba2_path)
        .arg(output_path);

    // On Windows, hide the console window to prevent flickering
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd.output().await.map_err(|e| BA2Error::ExtractionFailed {
        path: ba2_path.to_path_buf(),
        reason: format!("Failed to spawn BSArch.exe: {e}"),
    })?;

    // Check if extraction was successful
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BA2Error::ExtractionFailed {
            path: ba2_path.to_path_buf(),
            reason: format!("BSArch.exe failed: {stderr}"),
        }
        .into());
    }

    Ok(())
}

/// Extract multiple BA2 files with progress reporting and parallelism
///
/// # Arguments
///
/// * `files` - List of file entries to extract
/// * `config` - Application configuration (for `BSArch` path)
/// * `progress_tx` - Optional channel for progress updates
///
/// # Returns
///
/// `ExtractionResult` with details about successful and failed extractions
///
pub async fn extract_all(
    files: Vec<FileEntry>,
    config: AppConfig,
    progress_tx: Option<mpsc::Sender<ExtractionProgress>>,
) -> Result<ExtractionResult> {
    let total = files.len();

    // Use external BA2 tool if specified, otherwise use bundled BSArch.exe
    let bsarch_path = if config.advanced.ext_ba2_exe.is_empty() {
        // Default to bundled version in the same directory as the executable
        match std::env::current_exe() {
            Ok(exe_path) => exe_path
                .parent()
                .map(|p| p.join("BSArch.exe"))
                .unwrap_or_else(|| PathBuf::from("BSArch.exe")),
            Err(_) => PathBuf::from("BSArch.exe"),
        }
    } else {
        PathBuf::from(&config.advanced.ext_ba2_exe)
    };

    // Determine concurrency limit
    // Use number of logical cores, capped between 1 and 8 to avoid resource exhaustion
    let concurrency_limit = std::thread::available_parallelism()
        .map(std::num::NonZero::get)
        .unwrap_or(4)
        .clamp(1, 8);
    
    tracing::debug!("Extracting with concurrency limit: {}", concurrency_limit);

    let semaphore = Arc::new(Semaphore::new(concurrency_limit));
    let current_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    // Create a stream of extraction futures
    let results: Vec<FileExtractionResult> = stream::iter(files)
        .map(|file_entry| {
            let bsarch_path = bsarch_path.clone();
            let progress_tx = progress_tx.clone();
            let semaphore = semaphore.clone();
            let current_counter = current_counter.clone();
            
            // We must clone the data we need before the async block
            let file_path = file_entry.full_path.clone();
            let file_name = file_entry.file_name;
            
            async move {
                // Acquire permit to limit concurrency
                let Ok(_permit) = semaphore.acquire().await else {
                    // Semaphore was closed unexpectedly - treat as extraction failure
                    return FileExtractionResult {
                        file_path: file_path.clone(),
                        success: false,
                        error: Some("Extraction semaphore was closed unexpectedly".to_string()),
                    };
                };

                let current = current_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

                // Send started progress
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(ExtractionProgress::Started {
                        file_name: file_name.clone(),
                        current,
                        total,
                    }).await;
                }

                // Perform extraction
                let extraction_result = match extract_ba2_file(&file_path, None, &bsarch_path).await {
                    Ok(()) => FileExtractionResult {
                        file_path: file_path.clone(),
                        success: true,
                        error: None,
                    },
                    Err(e) => FileExtractionResult {
                        file_path: file_path.clone(),
                        success: false,
                        error: Some(e.to_string()),
                    },
                };

                // Send completed progress
                if let Some(ref tx) = progress_tx {
                    let _ = tx.send(ExtractionProgress::Completed {
                        file_name: file_name.clone(),
                        success: extraction_result.success,
                        error: extraction_result.error.clone(),
                    }).await;
                }

                extraction_result
            }
        })
        .buffer_unordered(concurrency_limit) // Run up to concurrency_limit futures in parallel
        .collect()
        .await;

    // Aggregate results
    let mut final_result = ExtractionResult::new();
    for res in results {
        final_result.add_result(res);
    }

    // Send final progress update
    if let Some(ref tx) = progress_tx {
        let _ = tx
            .send(ExtractionProgress::Finished {
                successful: final_result.successful,
                failed: final_result.failed,
            })
            .await;
    }

    Ok(final_result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extraction_result_creation() {
        let result = ExtractionResult::new();
        assert_eq!(result.successful, 0);
        assert_eq!(result.failed, 0);
        assert!(result.file_results.is_empty());
    }

    #[test]
    fn test_extraction_result_add_success() {
        let mut result = ExtractionResult::new();
        result.add_result(FileExtractionResult {
            file_path: PathBuf::from("/test/file.ba2"),
            success: true,
            error: None,
        });

        assert_eq!(result.successful, 1);
        assert_eq!(result.failed, 0);
        assert_eq!(result.file_results.len(), 1);
    }

    #[test]
    fn test_extraction_result_add_failure() {
        let mut result = ExtractionResult::new();
        result.add_result(FileExtractionResult {
            file_path: PathBuf::from("/test/file.ba2"),
            success: false,
            error: Some("Test error".to_string()),
        });

        assert_eq!(result.successful, 0);
        assert_eq!(result.failed, 1);
        assert_eq!(result.file_results.len(), 1);
    }

    #[test]
    fn test_extraction_result_filtering() {
        let mut result = ExtractionResult::new();

        result.add_result(FileExtractionResult {
            file_path: PathBuf::from("/test/success.ba2"),
            success: true,
            error: None,
        });

        result.add_result(FileExtractionResult {
            file_path: PathBuf::from("/test/failure.ba2"),
            success: false,
            error: Some("Error".to_string()),
        });

        let successful = result.successful_files();
        let failed = result.failed_files();

        assert_eq!(successful.len(), 1);
        assert_eq!(failed.len(), 1);
        assert_eq!(
            successful[0].file_name().unwrap().to_str().unwrap(),
            "success.ba2"
        );
        assert_eq!(
            failed[0].file_name().unwrap().to_str().unwrap(),
            "failure.ba2"
        );
    }

    #[tokio::test]
    async fn test_extract_ba2_file_not_found() {
        let result = extract_ba2_file(
            Path::new("/nonexistent/file.ba2"),
            None,
            Path::new("/fake/bsarch.exe"),
        )
        .await;

        assert!(result.is_err());
        // Should fail with ExtractionFailed error since file doesn't exist
        match result {
            Err(crate::error::Error::BA2(BA2Error::ExtractionFailed { .. })) => {
                // Expected error type
            }
            _ => panic!("Expected BA2Error::ExtractionFailed error"),
        }
    }
}
