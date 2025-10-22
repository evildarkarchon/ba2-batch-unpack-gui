//! UI module for Slint integration
//!
//! This module handles:
//! - Slint UI initialization and lifecycle
//! - UI callbacks and event handling
//! - State management between Rust backend and Slint frontend
//! - Slint + Tokio integration via async-compat

use crate::config::AppConfig;
use crate::models::{FileEntry, FileEntryList, SortBy};
use crate::operations::{extract_all, scan_for_ba2, ExtractionProgress, ScanProgress};
use anyhow::Result;
use humansize::{format_size, BINARY};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Include the generated Slint code
slint::include_modules!();

/// Initialize and run the UI
///
/// This function creates the main window and runs the Slint event loop.
/// It handles the integration between Slint's event loop and async operations.
///
/// # Example
///
/// ```no_run
/// use unpackrr::ui;
///
/// fn main() -> anyhow::Result<()> {
///     ui::run()?;
///     Ok(())
/// }
/// ```
pub fn run() -> Result<()> {
    // Create the main window
    let main_window = MainWindow::new()?;

    // Set up callbacks and state (to be implemented in Phase 1.8)
    setup_callbacks(&main_window);

    // Run the Slint event loop
    main_window.run()?;

    Ok(())
}

/// Application state shared between UI and background tasks
#[derive(Clone)]
struct AppState {
    config: AppConfig,
    file_entries: FileEntryList,
}

impl AppState {
    fn new() -> Result<Self> {
        let config = AppConfig::load()?;
        Ok(Self {
            config,
            file_entries: FileEntryList::new(),
        })
    }
}

/// Set up UI callbacks
///
/// This function wires up all the callbacks between the UI and backend logic.
/// It handles folder selection, scanning, extraction, and sorting.
fn setup_callbacks(main_window: &MainWindow) {
    // Load application state
    let state = match AppState::new() {
        Ok(s) => Arc::new(Mutex::new(s)),
        Err(e) => {
            tracing::error!("Failed to load configuration: {}", e);
            // Use default config if loading fails
            Arc::new(Mutex::new(AppState {
                config: AppConfig::default(),
                file_entries: FileEntryList::new(),
            }))
        }
    };

    setup_browse_folder_callback(main_window, Arc::clone(&state));
    setup_scan_callback(main_window, Arc::clone(&state));
    setup_extraction_callback(main_window, Arc::clone(&state));
    setup_sort_callback(main_window, Arc::clone(&state));

    tracing::info!("UI callbacks initialized");
}

/// Set up browse folder callback
fn setup_browse_folder_callback(main_window: &MainWindow, state: Arc<Mutex<AppState>>) {
    let weak = main_window.as_weak();

    main_window.on_browse_folder(move || {
        let weak_clone = weak.clone();
        let state = Arc::clone(&state);

        // Use rfd for native folder picker
        std::thread::spawn(move || {
            tracing::debug!("Opening folder picker dialog");
            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                let folder_str = folder.to_string_lossy().to_string();
                tracing::info!("User selected folder: {}", folder_str);

                // Update UI on main thread
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak_clone.upgrade() {
                        ui.set_selected_folder(SharedString::from(folder_str.clone()));

                        // Save last used directory
                        if let Ok(mut app_state) = state.lock() {
                            app_state.config.saved.directory = folder_str.clone();
                            if let Err(e) = app_state.config.save() {
                                tracing::error!("Failed to save configuration: {}", e);
                            } else {
                                tracing::debug!("Saved last used directory to config");
                            }
                        }
                    }
                });
            } else {
                tracing::debug!("Folder picker canceled by user");
            }
        });
    });
}

/// Set up scan callback
fn setup_scan_callback(main_window: &MainWindow, state: Arc<Mutex<AppState>>) {
    let weak = main_window.as_weak();

    main_window.on_start_scan(move || {
        let weak_clone = weak.clone();
        let state_clone = Arc::clone(&state);

        // Get selected folder from UI
        let folder = if let Some(ui) = weak.upgrade() {
            ui.get_selected_folder().to_string()
        } else {
            return;
        };

        if folder.is_empty() {
            tracing::warn!("Scan requested but no folder selected");
            return;
        }

        tracing::info!("Starting BA2 scan in: {}", folder);

        // Set scanning state
        if let Some(ui) = weak.upgrade() {
            ui.set_scanning(true);
            ui.set_status_text(SharedString::from("Scanning for BA2 files..."));
        }

        // Run scan in background thread with Tokio runtime
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let path = PathBuf::from(&folder);
                let (tx, mut rx) = mpsc::channel(100);

                // Get config
                let config = {
                    let app_state = state_clone.lock().unwrap();
                    app_state.config.clone()
                };

                // Spawn scan task
                let scan_task = tokio::spawn(async move {
                    scan_for_ba2(&path, &config, Some(tx)).await
                });

                // Process progress updates
                while let Some(progress) = rx.recv().await {
                    let weak = weak_clone.clone();
                    let status = match progress {
                        ScanProgress::Started { total_dirs } => {
                            format!("Starting scan of {} directories...", total_dirs)
                        }
                        ScanProgress::ScanningFolder {
                            folder,
                            current,
                            total,
                        } => {
                            format!("Scanning {} ({}/{})", folder, current, total)
                        }
                        ScanProgress::FoundBA2 { file_name } => {
                            format!("Found: {}", file_name)
                        }
                        ScanProgress::Complete { total_files } => {
                            format!("Scan complete: {} files found", total_files)
                        }
                    };

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak.upgrade() {
                            ui.set_status_text(SharedString::from(status));
                        }
                    });
                }

                // Get scan results
                match scan_task.await {
                    Ok(Ok(files)) => {
                        let total_files = files.len();
                        let total_size = files.iter().map(|f| f.file_size).sum::<u64>();

                        tracing::info!(
                            "Scan complete: found {} BA2 files, total size: {} bytes",
                            total_files,
                            total_size
                        );

                        // Convert to FileEntry and store in state
                        let entries: Vec<FileEntry> = files.into_iter().map(FileEntry::from).collect();

                        let corrupted_count = entries.iter().filter(|e| e.is_corrupted()).count();
                        if corrupted_count > 0 {
                            tracing::warn!("Found {} corrupted BA2 files", corrupted_count);
                        }

                        // Convert to FileRowData for UI
                        let row_data: Vec<FileRowData> = entries
                            .iter()
                            .map(|e| FileRowData {
                                file_name: SharedString::from(&e.file_name),
                                file_size: SharedString::from(e.size_display()),
                                num_files: SharedString::from(e.file_count_display()),
                                mod_name: SharedString::from(e.mod_display()),
                                is_bad: e.is_corrupted(),
                            })
                            .collect();

                        // Update state
                        {
                            let mut app_state = state_clone.lock().unwrap();
                            app_state.file_entries = FileEntryList::from_vec(entries);
                        }

                        // Update UI
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                ui.set_file_list(ModelRc::new(VecModel::from(row_data)));
                                ui.set_total_files(total_files as i32);
                                ui.set_total_size(SharedString::from(format_size(
                                    total_size,
                                    BINARY,
                                )));
                                ui.set_scanning(false);
                                ui.set_status_text(SharedString::from(format!(
                                    "Ready - {} files found",
                                    total_files
                                )));
                            }
                        });
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("Scan failed: {}", e);
                        tracing::error!("{}", error_msg);

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                ui.set_scanning(false);
                                ui.set_status_text(SharedString::from(error_msg));
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Scan task failed: {}", e);

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                ui.set_scanning(false);
                                ui.set_status_text(SharedString::from("Scan task failed"));
                            }
                        });
                    }
                }
            });
        });
    });
}

/// Set up extraction callback
fn setup_extraction_callback(main_window: &MainWindow, state: Arc<Mutex<AppState>>) {
    let weak = main_window.as_weak();

    main_window.on_start_extraction(move || {
        let weak_clone = weak.clone();
        let state_clone = Arc::clone(&state);

        // Set extracting state
        if let Some(ui) = weak.upgrade() {
            ui.set_extracting(true);
            ui.set_status_text(SharedString::from("Starting extraction..."));
        }

        // Run extraction in background thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let (tx, mut rx) = mpsc::channel(100);

                // Get files and config from state
                let (files, config) = {
                    let app_state = state_clone.lock().unwrap();
                    (
                        app_state.file_entries.entries().to_vec(),
                        app_state.config.clone(),
                    )
                };

                tracing::info!("Starting extraction of {} BA2 files", files.len());

                // Spawn extraction task
                let extract_task = tokio::spawn(async move {
                    extract_all(&files, &config, Some(tx)).await
                });

                // Process progress updates
                while let Some(progress) = rx.recv().await {
                    let weak = weak_clone.clone();
                    let status = match progress {
                        ExtractionProgress::Started {
                            file_name,
                            current,
                            total,
                        } => {
                            format!("Extracting {} ({}/{})", file_name, current, total)
                        }
                        ExtractionProgress::Completed {
                            file_name,
                            success,
                            error,
                        } => {
                            if success {
                                format!("Completed: {}", file_name)
                            } else {
                                format!(
                                    "Failed: {} - {}",
                                    file_name,
                                    error.unwrap_or_else(|| "Unknown error".to_string())
                                )
                            }
                        }
                        ExtractionProgress::Finished {
                            successful,
                            failed,
                        } => {
                            format!(
                                "Extraction complete: {} successful, {} failed",
                                successful, failed
                            )
                        }
                    };

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak.upgrade() {
                            ui.set_status_text(SharedString::from(status));
                        }
                    });
                }

                // Get extraction results
                match extract_task.await {
                    Ok(Ok(result)) => {
                        tracing::info!(
                            "Extraction complete: {} successful, {} failed",
                            result.successful,
                            result.failed
                        );

                        if result.failed > 0 {
                            tracing::warn!(
                                "Failed files: {:?}",
                                result
                                    .failed_files()
                                    .iter()
                                    .map(|p| p.display().to_string())
                                    .collect::<Vec<_>>()
                            );
                        }

                        let final_status = format!(
                            "Extraction complete: {} successful, {} failed",
                            result.successful, result.failed
                        );

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                ui.set_extracting(false);
                                ui.set_status_text(SharedString::from(final_status));
                            }
                        });
                    }
                    Ok(Err(e)) => {
                        let error_msg = format!("Extraction failed: {}", e);
                        tracing::error!("{}", error_msg);

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                ui.set_extracting(false);
                                ui.set_status_text(SharedString::from(error_msg));
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Extraction task failed: {}", e);

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                ui.set_extracting(false);
                                ui.set_status_text(SharedString::from("Extraction task failed"));
                            }
                        });
                    }
                }
            });
        });
    });
}

/// Set up sort callback
fn setup_sort_callback(main_window: &MainWindow, state: Arc<Mutex<AppState>>) {
    let weak = main_window.as_weak();

    main_window.on_sort_by_column(move |column| {
        let sort_by = match column {
            0 => SortBy::Name,
            1 => SortBy::Size,
            2 => SortBy::FileCount,
            3 => SortBy::ModName,
            _ => return,
        };

        // Sort entries in state
        {
            let mut app_state = state.lock().unwrap();
            app_state.file_entries.sort_by(sort_by);
        }

        // Update UI
        let state_clone = Arc::clone(&state);
        let weak_clone = weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = weak_clone.upgrade() {
                let app_state = state_clone.lock().unwrap();
                let row_data: Vec<FileRowData> = app_state
                    .file_entries
                    .entries()
                    .iter()
                    .map(|e| FileRowData {
                        file_name: SharedString::from(&e.file_name),
                        file_size: SharedString::from(e.size_display()),
                        num_files: SharedString::from(e.file_count_display()),
                        mod_name: SharedString::from(e.mod_display()),
                        is_bad: e.is_corrupted(),
                    })
                    .collect();

                ui.set_file_list(ModelRc::new(VecModel::from(row_data)));
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slint_module_exists() {
        // This test verifies that the Slint code was successfully compiled
        // We can't actually run the UI in tests, but we can verify it compiles
        assert!(true, "Slint module compiled successfully");
    }
}
