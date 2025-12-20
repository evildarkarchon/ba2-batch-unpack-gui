//! BA2 file scanning operations
//!
//! This module provides directory scanning functionality for discovering BA2 files
//! in a directory structure. It follows the Python version's logic of scanning
//! second-tier directories (mod folders) to avoid scanning BA2 files that won't
//! be loaded by the game.

use crate::ba2::BA2Header;
use crate::config::AppConfig;
use crate::error::{Result, ValidationError};
use crate::operations::BA2FileInfo;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::{debug, warn};

/// Progress update for scanning operations
#[derive(Debug, Clone)]
pub enum ScanProgress {
    /// Started scanning a directory
    Started {
        /// Total number of directories to scan
        total_dirs: usize,
    },

    /// Scanning a specific mod folder
    ScanningFolder {
        /// Name of the folder being scanned
        folder: String,
        /// Current directory index
        current: usize,
        /// Total number of directories
        total: usize,
    },

    /// Found a BA2 file
    FoundBA2 {
        /// Name of the BA2 file found
        file_name: String,
    },

    /// Finished scanning
    Complete {
        /// Total number of BA2 files discovered
        total_files: usize,
    },
}

/// Scan a directory for BA2 files matching the configured postfixes
///
/// This function scans second-tier directories (mod folders) for BA2 files.
/// It filters files based on:
/// - Postfix patterns (e.g., "_main", "_textures")
/// - Ignored file patterns (exact, substring, regex)
/// - File validity (corrupt BA2 files are marked as bad)
///
/// # Arguments
///
/// * `path` - The root directory to scan (typically the Fallout 4 Data folder)
/// * `config` - Application configuration containing postfixes and ignored patterns
/// * `progress_tx` - Optional channel for sending progress updates
///
/// # Returns
///
/// A vector of `BA2FileInfo` structs for each discovered BA2 file
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use unpackrr::operations::scan::scan_for_ba2;
/// use unpackrr::config::AppConfig;
///
/// # async fn example() -> anyhow::Result<()> {
/// let config = AppConfig::load()?;
/// let path = Path::new("C:/Games/Fallout4/Data");
/// let files = scan_for_ba2(path, &config, None).await?;
/// println!("Found {} BA2 files", files.len());
/// # Ok(())
/// # }
/// ```
pub async fn scan_for_ba2(
    path: &Path,
    config: &AppConfig,
    progress_tx: Option<mpsc::Sender<ScanProgress>>,
) -> Result<Vec<BA2FileInfo>> {
    debug!("Starting BA2 scan in: {}", path.display());

    // Verify the path exists and is a directory
    if !path.exists() {
        return Err(ValidationError::PathNotFound(path.to_path_buf()).into());
    }

    if !path.is_dir() {
        return Err(ValidationError::NotADirectory(path.to_path_buf()).into());
    }

    // List all first-tier directories (mod folders)
    let entries = fs::read_dir(path).map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("Failed to read directory {}: {}", path.display(), e),
        )
    })?;

    let mut mod_folders: Vec<PathBuf> = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip files, only process directories
        if path.is_dir() {
            mod_folders.push(path);
        }
    }

    let total_folders = mod_folders.len();
    debug!("Found {} mod folders to scan", total_folders);

    // Send started progress
    if let Some(ref tx) = progress_tx {
        let _ = tx
            .send(ScanProgress::Started {
                total_dirs: total_folders,
            })
            .await;
    }

    // Use rayon for parallel scanning of mod folders
    // Wrap in spawn_blocking to avoid blocking the async executor
    // Note: Progress updates during parallel scanning are omitted to avoid
    // tokio/rayon runtime conflicts. Only start and complete messages are sent.
    let config_clone = config.clone();
    let all_ba2: Vec<BA2FileInfo> = tokio::task::spawn_blocking(move || {
        mod_folders
            .into_par_iter()
            .flat_map(|mod_folder| scan_mod_folder(&mod_folder, &config_clone))
            .collect()
    })
    .await
    .map_err(|e| std::io::Error::other(format!("Scan task failed: {e}")))?;

    // Send completion progress
    if let Some(ref tx) = progress_tx {
        let _ = tx
            .send(ScanProgress::Complete {
                total_files: all_ba2.len(),
            })
            .await;
    }

    debug!("Scan complete. Found {} BA2 files", all_ba2.len());
    Ok(all_ba2)
}

/// Scan a single mod folder for BA2 files
fn scan_mod_folder(mod_folder: &Path, config: &AppConfig) -> Vec<BA2FileInfo> {
    let mut ba2_files = Vec::new();

    let dir_name = mod_folder
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // List all files in the mod folder
    let entries = match fs::read_dir(mod_folder) {
        Ok(entries) => entries,
        Err(e) => {
            warn!("Failed to read mod folder {}: {}", mod_folder.display(), e);
            return ba2_files;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Only process .ba2 files
        if path.extension().and_then(|e| e.to_str()) != Some("ba2") {
            continue;
        }

        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Check if file matches postfix patterns
        let file_name_lower = file_name.to_lowercase();
        let matches_postfix = config
            .extraction
            .postfixes
            .iter()
            .any(|postfix| file_name_lower.contains(&postfix.to_lowercase()));

        if !matches_postfix {
            debug!("Skipping {} (doesn't match postfix patterns)", file_name);
            continue;
        }

        // Check if file should be ignored
        if config.should_ignore_file(&path) {
            debug!("Skipping {} (matches ignored pattern)", file_name);
            continue;
        }

        // Get file size
        let file_size = match fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                warn!("Failed to get metadata for {}: {}", path.display(), e);
                0
            }
        };

        // Try to read BA2 header to get file count and validate
        let (num_files, is_bad) = match BA2Header::parse(&path) {
            Ok(header) => (header.file_count, false),
            Err(e) => {
                warn!("Failed to parse BA2 header for {}: {}", path.display(), e);
                (0, true)
            }
        };

        ba2_files.push(BA2FileInfo {
            file_name,
            file_size,
            num_files,
            dir_name: dir_name.clone(),
            full_path: path,
            is_bad,
        });
    }

    ba2_files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Create a test directory structure with BA2 files
    fn create_test_structure() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let data_path = temp_dir.path().to_path_buf();

        // Create mod folders
        let mod1 = data_path.join("TestMod1");
        let mod2 = data_path.join("TestMod2");
        fs::create_dir(&mod1).unwrap();
        fs::create_dir(&mod2).unwrap();

        // Create test BA2 files with valid headers
        let ba2_1 = mod1.join("TestMod1_Main.ba2");
        let ba2_2 = mod1.join("TestMod1_Textures.ba2");
        let ba2_3 = mod2.join("TestMod2_Main.ba2");
        let ba2_ignored = mod2.join("TestMod2_Sounds.ba2"); // Won't match postfix

        for path in &[&ba2_1, &ba2_2, &ba2_3, &ba2_ignored] {
            create_test_ba2(path, 10);
        }

        // Create a non-BA2 file that should be ignored
        let _txt_file = mod1.join("readme.txt");
        File::create(&_txt_file).unwrap();

        (temp_dir, data_path)
    }

    /// Create a test BA2 file with a valid header
    fn create_test_ba2(path: &Path, file_count: u32) {
        let mut file = File::create(path).unwrap();

        // Write BA2 header
        file.write_all(b"BTDX").unwrap(); // Magic
        file.write_all(&1u32.to_le_bytes()).unwrap(); // Version
        file.write_all(b"GNRL").unwrap(); // Type
        file.write_all(&file_count.to_le_bytes()).unwrap(); // File count
        file.write_all(&0u64.to_le_bytes()).unwrap(); // Names offset

        // Write some dummy data to make the file look real
        file.write_all(&vec![0u8; 100]).unwrap();
    }

    #[tokio::test]
    async fn test_scan_for_ba2_basic() {
        let (_temp_dir, data_path) = create_test_structure();

        let mut config = AppConfig::default();
        config.extraction.postfixes = vec!["_main".to_string(), "_textures".to_string()];

        let result = scan_for_ba2(&data_path, &config, None).await;
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), 3); // Should find 3 BA2 files matching postfixes

        // Verify file names
        let file_names: Vec<String> = files.iter().map(|f| f.file_name.clone()).collect();
        assert!(file_names.contains(&"TestMod1_Main.ba2".to_string()));
        assert!(file_names.contains(&"TestMod1_Textures.ba2".to_string()));
        assert!(file_names.contains(&"TestMod2_Main.ba2".to_string()));
    }

    #[tokio::test]
    async fn test_scan_for_ba2_with_ignored() {
        let (_temp_dir, data_path) = create_test_structure();

        let mut config = AppConfig::default();
        config.extraction.postfixes = vec!["_main".to_string(), "_textures".to_string()];
        config.extraction.ignored_files = vec!["TestMod1_Main.ba2".to_string()];

        let result = scan_for_ba2(&data_path, &config, None).await;
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), 2); // Should find 2 files (one ignored)

        let file_names: Vec<String> = files.iter().map(|f| f.file_name.clone()).collect();
        assert!(!file_names.contains(&"TestMod1_Main.ba2".to_string()));
        assert!(file_names.contains(&"TestMod1_Textures.ba2".to_string()));
        assert!(file_names.contains(&"TestMod2_Main.ba2".to_string()));
    }

    #[tokio::test]
    async fn test_scan_for_ba2_progress() {
        let (_temp_dir, data_path) = create_test_structure();

        let mut config = AppConfig::default();
        config.extraction.postfixes = vec!["_main".to_string()];

        let (tx, mut rx) = mpsc::channel(100);

        // Run scan in background task
        let scan_task =
            tokio::spawn(async move { scan_for_ba2(&data_path, &config, Some(tx)).await });

        // Collect progress updates
        let mut progress_updates = Vec::new();
        while let Some(progress) = rx.recv().await {
            progress_updates.push(progress);
        }

        let result = scan_task.await.unwrap();
        assert!(result.is_ok());

        // Verify we got progress updates
        assert!(!progress_updates.is_empty());

        // Check for Started and Complete messages
        assert!(
            progress_updates
                .iter()
                .any(|p| matches!(p, ScanProgress::Started { .. }))
        );
        assert!(
            progress_updates
                .iter()
                .any(|p| matches!(p, ScanProgress::Complete { .. }))
        );
    }

    #[tokio::test]
    async fn test_scan_nonexistent_path() {
        let config = AppConfig::default();
        let result = scan_for_ba2(Path::new("/nonexistent/path"), &config, None).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_mod_folder_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = AppConfig::default();

        let result = scan_mod_folder(temp_dir.path(), &config);
        assert_eq!(result.len(), 0);
    }
}
