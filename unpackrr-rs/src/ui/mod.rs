//! UI module for Slint integration
//!
//! This module handles:
//! - Slint UI initialization and lifecycle
//! - UI callbacks and event handling
//! - State management between Rust backend and Slint frontend
//! - Slint + Tokio integration via async-compat

pub mod notifications;

use crate::config::AppConfig;
use crate::models::{FileEntry, FileEntryList, SortBy};
use crate::operations::{extract_all, scan_for_ba2, ExtractionProgress, ScanProgress};
use anyhow::Result;
use humansize::{format_size, BINARY};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

// Include the generated Slint code
slint::include_modules!();

// Re-export notification types for convenience
pub use notifications::{show_dialog, show_toast, DialogConfig, ToastData};

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

/// Control signals for extraction (Phase 2.3)
#[derive(Debug, Clone)]
enum ExtractionControl {
    Pause,
    Resume,
    Cancel,
}

/// Extraction control state (Phase 2.3)
struct ExtractionControlState {
    control_tx: Option<tokio::sync::mpsc::UnboundedSender<ExtractionControl>>,
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

    // Phase 2.3: Create extraction control state
    let extraction_control = Arc::new(Mutex::new(ExtractionControlState {
        control_tx: None,
    }));

    setup_browse_folder_callback(main_window, Arc::clone(&state));
    setup_scan_callback(main_window, Arc::clone(&state));
    setup_extraction_callback(main_window, Arc::clone(&state), Arc::clone(&extraction_control));
    setup_sort_callback(main_window, Arc::clone(&state));
    setup_threshold_callbacks(main_window, Arc::clone(&state)); // Phase 2.3
    setup_file_actions_callback(main_window, Arc::clone(&state)); // Phase 2.3
    setup_open_folder_callback(main_window, Arc::clone(&state)); // Phase 2.3
    setup_extraction_control_callbacks(main_window, Arc::clone(&extraction_control)); // Phase 2.3
    setup_update_checker_callback(main_window);
    setup_platform_integration(main_window, Arc::clone(&state)); // Phase 2.9

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
fn setup_extraction_callback(
    main_window: &MainWindow,
    state: Arc<Mutex<AppState>>,
    extraction_control: Arc<Mutex<ExtractionControlState>>,
) {
    let weak = main_window.as_weak();

    main_window.on_start_extraction(move || {
        let weak_clone = weak.clone();
        let state_clone = Arc::clone(&state);
        let extraction_control_clone = Arc::clone(&extraction_control);

        // Set extracting state
        if let Some(ui) = weak.upgrade() {
            ui.set_extracting(true);
            ui.set_extraction_complete(false); // Phase 2.3: Reset completion state
            ui.set_paused(false); // Phase 2.3: Reset pause state
            ui.set_status_text(SharedString::from("Starting extraction..."));
        }

        // Run extraction in background thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let (tx, mut rx) = mpsc::channel(100);

                // Phase 2.3: Create control channel
                let (control_tx, mut control_rx) = tokio::sync::mpsc::unbounded_channel();

                // Phase 2.3: Store control sender in shared state
                {
                    let mut ctrl_state = extraction_control_clone.lock().unwrap();
                    ctrl_state.control_tx = Some(control_tx);
                }

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

                // Phase 2.3: Track pause state
                let mut is_paused = false;
                let mut should_cancel = false;

                // Phase 2.3: Track extraction timing for speed/ETA calculation
                let extraction_start_time = std::time::Instant::now();
                let mut last_update_time = std::time::Instant::now();

                // Process progress updates and control signals
                loop {
                    tokio::select! {
                        // Handle extraction progress
                        progress_opt = rx.recv() => {
                            // Check if progress channel closed
                            let Some(progress) = progress_opt else {
                                tracing::info!("Progress channel closed, extraction finished");
                                break;
                            };
                            // Check if we should cancel
                            if should_cancel {
                                tracing::info!("Cancelling extraction...");
                                break;
                            }

                            // Wait while paused
                            while is_paused && !should_cancel {
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                                // Check for control signals while paused
                                if let Ok(control) = control_rx.try_recv() {
                                    match control {
                                        ExtractionControl::Resume => {
                                            tracing::info!("Resuming extraction");
                                            is_paused = false;
                                            let weak = weak_clone.clone();
                                            let _ = slint::invoke_from_event_loop(move || {
                                                if let Some(ui) = weak.upgrade() {
                                                    ui.set_paused(false);
                                                }
                                            });
                                        }
                                        ExtractionControl::Cancel => {
                                            should_cancel = true;
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                            }

                            if should_cancel {
                                break;
                            }

                    let weak = weak_clone.clone();
                    let status = match &progress {  // Changed to &progress to avoid move
                        ExtractionProgress::Started {
                            file_name,
                            current,
                            total,
                        } => {
                            // Phase 2.3: Update progress properties in UI
                            let file_name_clone = file_name.clone();
                            let current_val = *current;
                            let total_val = *total;

                            // Phase 2.3: Calculate speed and ETA
                            let elapsed = extraction_start_time.elapsed();
                            let elapsed_secs = elapsed.as_secs_f64();

                            // Only update speed/ETA every second to avoid flickering
                            let should_update_timing = last_update_time.elapsed().as_secs() >= 1;
                            if should_update_timing {
                                last_update_time = std::time::Instant::now();
                            }

                            let speed_str = if should_update_timing && current_val > 0 && elapsed_secs > 0.0 {
                                let files_per_sec = current_val as f64 / elapsed_secs;
                                if files_per_sec >= 1.0 {
                                    format!("{:.1} files/s", files_per_sec)
                                } else {
                                    format!("{:.1} s/file", 1.0 / files_per_sec)
                                }
                            } else {
                                String::new()
                            };

                            let eta_str = if should_update_timing && current_val > 0 && elapsed_secs > 0.0 {
                                let remaining = total_val.saturating_sub(current_val);
                                if remaining > 0 {
                                    let avg_time_per_file = elapsed_secs / current_val as f64;
                                    let eta_secs = (remaining as f64 * avg_time_per_file) as u64;

                                    let hours = eta_secs / 3600;
                                    let mins = (eta_secs % 3600) / 60;
                                    let secs = eta_secs % 60;

                                    if hours > 0 {
                                        format!("{}h {}m", hours, mins)
                                    } else if mins > 0 {
                                        format!("{}m {}s", mins, secs)
                                    } else {
                                        format!("{}s", secs)
                                    }
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            let weak_progress = weak.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = weak_progress.upgrade() {
                                    ui.set_current_extracting_file(SharedString::from(file_name_clone));
                                    ui.set_current_file_index(current_val as i32);
                                    ui.set_total_extraction_files(total_val as i32);

                                    // Calculate progress percentage (avoid division by zero)
                                    let progress_pct = if total_val > 0 {
                                        ((current_val * 100) / total_val) as i32
                                    } else {
                                        0
                                    };
                                    ui.set_extraction_progress(progress_pct);

                                    // Phase 2.3: Update speed and ETA
                                    if should_update_timing {
                                        ui.set_extraction_speed(SharedString::from(speed_str));
                                        ui.set_extraction_eta(SharedString::from(eta_str));
                                    }
                                }
                            });

                            format!("Extracting {} ({}/{})", file_name, current, total)
                        }
                        ExtractionProgress::Completed {
                            file_name,
                            success,
                            error,
                        } => {
                            if *success {  // Dereference since we're now matching on &progress
                                format!("Completed: {}", file_name)
                            } else {
                                format!(
                                    "Failed: {} - {}",
                                    file_name,
                                    error.as_ref().map(|s| s.as_str()).unwrap_or("Unknown error")
                                )
                            }
                        }
                        ExtractionProgress::Finished {
                            successful,
                            failed,
                        } => {
                            // Phase 2.3: Reset progress properties
                            let weak_progress = weak.clone();
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = weak_progress.upgrade() {
                                    ui.set_current_extracting_file(SharedString::from(""));
                                    ui.set_current_file_index(0);
                                    ui.set_total_extraction_files(0);
                                    ui.set_extraction_progress(0);
                                    ui.set_extraction_speed(SharedString::from("")); // Phase 2.3: Reset speed
                                    ui.set_extraction_eta(SharedString::from("")); // Phase 2.3: Reset ETA
                                }
                            });

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
                        } // End of Some(progress) arm

                        // Handle control signals
                        Some(control) = control_rx.recv() => {
                            tracing::info!("Received control signal: {:?}", control);
                            match control {
                                ExtractionControl::Pause => {
                                    tracing::info!("Pausing extraction");
                                    is_paused = true;
                                    let weak = weak_clone.clone();
                                    let _ = slint::invoke_from_event_loop(move || {
                                        if let Some(ui) = weak.upgrade() {
                                            ui.set_paused(true);
                                            ui.set_status_text(SharedString::from("Extraction paused"));
                                        }
                                    });
                                }
                                ExtractionControl::Resume => {
                                    tracing::info!("Resuming extraction");
                                    is_paused = false;
                                    let weak = weak_clone.clone();
                                    let _ = slint::invoke_from_event_loop(move || {
                                        if let Some(ui) = weak.upgrade() {
                                            ui.set_paused(false);
                                            ui.set_status_text(SharedString::from("Extraction resumed"));
                                        }
                                    });
                                }
                                ExtractionControl::Cancel => {
                                    tracing::info!("Cancelling extraction");
                                    let weak = weak_clone.clone();
                                    let _ = slint::invoke_from_event_loop(move || {
                                        if let Some(ui) = weak.upgrade() {
                                            ui.set_status_text(SharedString::from("Extraction cancelled"));
                                        }
                                    });
                                    break;
                                }
                            }
                        }
                    } // End of tokio::select!

                    // Check if we should break (control signals or extraction finished)
                    if should_cancel {
                        break;
                    }
                } // End of loop

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

                        // Phase 2.3: Get extraction path for "Open Folder" button
                        let extraction_path = {
                            let app_state = state_clone.lock().unwrap();
                            app_state.config.advanced.extraction_path.clone()
                        };

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                ui.set_extracting(false);
                                ui.set_status_text(SharedString::from(final_status));

                                // Phase 2.3: Show "Open Folder" button after successful extraction
                                if result.successful > 0 {
                                    ui.set_extraction_complete(true);
                                    ui.set_extraction_folder(SharedString::from(extraction_path));
                                }
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

/// Set up extraction control callbacks (Phase 2.3)
fn setup_extraction_control_callbacks(
    main_window: &MainWindow,
    extraction_control: Arc<Mutex<ExtractionControlState>>,
) {
    // Pause extraction
    {
        let extraction_control_clone = Arc::clone(&extraction_control);
        main_window.on_pause_extraction(move || {
            tracing::info!("Pause extraction requested");
            let ctrl_state = extraction_control_clone.lock().unwrap();
            if let Some(tx) = &ctrl_state.control_tx {
                if let Err(e) = tx.send(ExtractionControl::Pause) {
                    tracing::error!("Failed to send pause signal: {}", e);
                }
            } else {
                tracing::warn!("No active extraction to pause");
            }
        });
    }

    // Resume extraction
    {
        let extraction_control_clone = Arc::clone(&extraction_control);
        main_window.on_resume_extraction(move || {
            tracing::info!("Resume extraction requested");
            let ctrl_state = extraction_control_clone.lock().unwrap();
            if let Some(tx) = &ctrl_state.control_tx {
                if let Err(e) = tx.send(ExtractionControl::Resume) {
                    tracing::error!("Failed to send resume signal: {}", e);
                }
            } else {
                tracing::warn!("No active extraction to resume");
            }
        });
    }

    // Cancel extraction
    {
        let extraction_control_clone = Arc::clone(&extraction_control);
        main_window.on_cancel_extraction(move || {
            tracing::info!("Cancel extraction requested");
            let ctrl_state = extraction_control_clone.lock().unwrap();
            if let Some(tx) = &ctrl_state.control_tx {
                if let Err(e) = tx.send(ExtractionControl::Cancel) {
                    tracing::error!("Failed to send cancel signal: {}", e);
                }
            } else {
                tracing::warn!("No active extraction to cancel");
            }
        });
    }
}

/// Set up update checker callback (Phase 2.6)
fn setup_update_checker_callback(main_window: &MainWindow) {
    let weak = main_window.as_weak();

    main_window.on_check_for_updates(move || {
        let weak_clone = weak.clone();

        tracing::info!("User requested update check");

        // Show toast notification that we're checking
        if let Some(ui) = weak.upgrade() {
            show_toast(&ui, ToastData {
                message: "Checking for updates...".to_string(),
                notification_type: NotificationType::Info,
                show: true,
            });
        }

        // Run update check in background thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                match crate::update_checker::check_for_updates().await {
                    Ok(Some(update_info)) => {
                        // Update available
                        tracing::info!(
                            "Update available: {} -> {}",
                            update_info.current_version,
                            update_info.latest_version
                        );

                        let download_url = update_info.download_url.clone();

                        // Open the download page in the browser immediately
                        if let Err(e) = open::that(&download_url) {
                            tracing::error!("Failed to open browser: {}", e);
                        }

                        let message = format!(
                            "Update available!\n\nCurrent version: {}\nLatest version: {}\n\nOpening download page in your browser...\n\n{}",
                            update_info.current_version,
                            update_info.latest_version,
                            if update_info.release_notes.len() > 200 {
                                format!("{}...", &update_info.release_notes[..200])
                            } else {
                                update_info.release_notes.clone()
                            }
                        );

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                show_dialog(&ui, DialogConfig {
                                    title: "Update Available".to_string(),
                                    message,
                                    dialog_type: NotificationType::Success,
                                    primary_button: "OK".to_string(),
                                    secondary_button: None,
                                });
                            }
                        });
                    }
                    Ok(None) => {
                        // Already up to date
                        tracing::info!("Already running the latest version");

                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                show_toast(&ui, ToastData {
                                    message: "You're running the latest version!".to_string(),
                                    notification_type: NotificationType::Success,
                                    show: true,
                                });
                            }
                        });
                    }
                    Err(e) => {
                        // Error checking for updates
                        tracing::error!("Failed to check for updates: {}", e);

                        let error_msg = format!("Failed to check for updates: {}", e);
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = weak_clone.upgrade() {
                                show_toast(&ui, ToastData {
                                    message: error_msg,
                                    notification_type: NotificationType::Error,
                                    show: true,
                                });
                            }
                        });
                    }
                }
            });
        });
    });
}

/// Set up platform integration (Phase 2.9)
///
/// Detects the default BA2 file handler on Windows and auto-populates
/// the external tool setting if it's empty.
fn setup_platform_integration(main_window: &MainWindow, state: Arc<Mutex<AppState>>) {
    tracing::info!("Initializing platform integration (Phase 2.9)");

    // Try to detect the default BA2 handler
    match crate::platform::get_default_ba2_handler() {
        Ok(Some(handler_path)) => {
            tracing::info!("Detected default BA2 handler: {}", handler_path.display());

            // Check if the external tool path is empty in config
            let should_populate = {
                let app_state = state.lock().unwrap();
                app_state.config.advanced.ext_ba2_exe.is_empty()
            };

            if should_populate {
                // Update the config with the detected handler
                let handler_str = handler_path.to_string_lossy().to_string();

                {
                    let mut app_state = state.lock().unwrap();
                    app_state.config.advanced.ext_ba2_exe = handler_str.clone();

                    // Save the updated config
                    if let Err(e) = app_state.config.save() {
                        tracing::error!("Failed to save config with auto-detected BA2 handler: {}", e);
                    } else {
                        tracing::info!("Auto-populated external BA2 tool: {}", handler_str);
                    }
                }

                // Update the UI
                main_window.set_settings_external_tool(SharedString::from(handler_str.clone()));

                // Show a toast notification
                show_toast(main_window, ToastData {
                    message: format!("Auto-detected BA2 handler: {}", handler_str),
                    notification_type: NotificationType::Info,
                    show: true,
                });
            } else {
                tracing::debug!("External BA2 tool already configured, skipping auto-detection");
            }
        }
        Ok(None) => {
            tracing::info!("No default BA2 handler detected (this is normal on non-Windows platforms)");
        }
        Err(e) => {
            tracing::warn!("Failed to detect default BA2 handler: {}", e);
        }
    }
}

/// Set up threshold filtering callbacks (Phase 2.3)
fn setup_threshold_callbacks(main_window: &MainWindow, state: Arc<Mutex<AppState>>) {
    let weak = main_window.as_weak();

    // Handle threshold value changes
    {
        let state_clone = Arc::clone(&state);
        let weak_clone = weak.clone();

        main_window.on_threshold_changed(move |value| {
            let value_str = value.to_string();

            if value_str.is_empty() {
                // Clear threshold - show all files
                let weak = weak_clone.clone();
                let state = Arc::clone(&state_clone);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak.upgrade() {
                        refresh_file_table(&ui, &state, None);
                    }
                });
                return;
            }

            // Parse the threshold value
            match crate::operations::parse_size(&value_str) {
                Ok(threshold_bytes) => {
                    tracing::info!("Threshold set to: {} bytes", threshold_bytes);

                    let weak = weak_clone.clone();
                    let state = Arc::clone(&state_clone);
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak.upgrade() {
                            refresh_file_table(&ui, &state, Some(threshold_bytes));
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("Invalid threshold value '{}': {}", value_str, e);
                }
            }
        });
    }

    // Handle auto-threshold toggle
    {
        let state_clone = Arc::clone(&state);
        let weak_clone = weak.clone();

        main_window.on_auto_threshold_toggled(move |enabled| {
            if enabled {
                // Calculate auto-threshold (235 file limit)
                let (entries_count, threshold_opt) = {
                    let app_state = state_clone.lock().unwrap();
                    let entries = app_state.file_entries.entries();
                    let count = entries.len();

                    if count <= 235 {
                        (count, None)
                    } else {
                        // Get the 235th largest file's size
                        let mut sorted_sizes: Vec<u64> = entries.iter()
                            .map(|e| e.file_size)
                            .collect();
                        sorted_sizes.sort_unstable();
                        sorted_sizes.reverse();

                        let threshold = sorted_sizes[234]; // 0-indexed, so 234 is the 235th item
                        (count, Some(threshold))
                    }
                };

                if let Some(threshold) = threshold_opt {
                    let threshold_str = format_size(threshold, BINARY);

                    tracing::info!(
                        "Auto-threshold calculated: {} ({} bytes) - will keep 235 files",
                        threshold_str,
                        threshold
                    );

                    let weak = weak_clone.clone();
                    let state = Arc::clone(&state_clone);
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak.upgrade() {
                            ui.set_threshold_value(SharedString::from(threshold_str.clone()));
                            refresh_file_table(&ui, &state, Some(threshold));

                            show_toast(&ui, ToastData {
                                message: format!(
                                    "Auto-threshold set to {} (keeping 235 files)",
                                    threshold_str
                                ),
                                notification_type: NotificationType::Success,
                                show: true,
                            });
                        }
                    });
                } else {
                    tracing::info!("Auto-threshold not needed: only {} files", entries_count);

                    let weak = weak_clone.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak.upgrade() {
                            ui.set_auto_threshold(false);
                            show_toast(&ui, ToastData {
                                message: format!(
                                    "Auto-threshold not needed: only {} BA2 files found (limit is 235)",
                                    entries_count
                                ),
                                notification_type: NotificationType::Info,
                                show: true,
                            });
                        }
                    });
                }
            } else {
                // Auto-threshold disabled - clear threshold
                let weak = weak_clone.clone();
                let state = Arc::clone(&state_clone);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak.upgrade() {
                        ui.set_threshold_value(SharedString::from(""));
                        refresh_file_table(&ui, &state, None);
                    }
                });
            }
        });
    }
}

/// Set up file actions callback (Phase 2.3 - ignore/open)
fn setup_file_actions_callback(main_window: &MainWindow, state: Arc<Mutex<AppState>>) {
    let weak = main_window.as_weak();

    main_window.on_file_action(move |row_index, action| {
        let action_str = action.to_string();
        tracing::info!("File action requested: {} for row {}", action_str, row_index);

        match action_str.as_str() {
            "ignore" => {
                // Get the file name from the row
                let file_name = if let Some(ui) = weak.upgrade() {
                    let file_list = ui.get_file_list();
                    if row_index >= 0 && (row_index as usize) < file_list.row_count() {
                        file_list.row_data(row_index as usize).unwrap().file_name.to_string()
                    } else {
                        tracing::error!("Invalid row index: {}", row_index);
                        return;
                    }
                } else {
                    return;
                };

                tracing::info!("Ignoring file: {}", file_name);

                // Add to ignored files in config
                {
                    let mut app_state = state.lock().unwrap();
                    // TODO: Add the file to the ignored list in config
                    // For now, just remove it from the current list
                    let entries = app_state.file_entries.entries().to_vec();
                    let filtered: Vec<FileEntry> = entries
                        .into_iter()
                        .filter(|e| e.file_name != file_name)
                        .collect();
                    app_state.file_entries = FileEntryList::from_vec(filtered);
                }

                // Refresh the table
                let weak_clone = weak.clone();
                let state_clone = Arc::clone(&state);
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak_clone.upgrade() {
                        refresh_file_table(&ui, &state_clone, None);

                        show_toast(&ui, ToastData {
                            message: format!("Ignored file: {}", file_name),
                            notification_type: NotificationType::Success,
                            show: true,
                        });
                    }
                });
            }
            "open" => {
                // Get the file info from state
                let (file_name, file_path, ext_tool_path) = {
                    let app_state = state.lock().unwrap();
                    let entries = app_state.file_entries.entries();

                    if row_index < 0 || (row_index as usize) >= entries.len() {
                        tracing::error!("Invalid row index: {}", row_index);
                        return;
                    }

                    let entry = &entries[row_index as usize];
                    (
                        entry.file_name.clone(),
                        entry.full_path.clone(),
                        app_state.config.advanced.ext_ba2_exe.clone(),
                    )
                };

                tracing::info!("Opening BA2 file with external tool: {}", file_path.display());

                // Check if file exists
                if !file_path.exists() {
                    tracing::error!("File not found: {}", file_path.display());
                    let weak_clone = weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak_clone.upgrade() {
                            show_toast(&ui, ToastData {
                                message: format!("File not found: {}", file_name),
                                notification_type: NotificationType::Error,
                                show: true,
                            });
                        }
                    });
                    return;
                }

                // Check if external tool is configured
                if ext_tool_path.is_empty() {
                    tracing::warn!("No external BA2 tool configured");
                    let weak_clone = weak.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = weak_clone.upgrade() {
                            show_toast(&ui, ToastData {
                                message: "No external BA2 tool configured.\nPlease set the tool path in Settings > Advanced.".to_string(),
                                notification_type: NotificationType::Warning,
                                show: true,
                            });
                        }
                    });
                    return;
                }

                // Launch external tool in background thread
                let weak_clone = weak.clone();
                std::thread::spawn(move || {
                    use std::process::Command;

                    tracing::info!("Launching: {} {}", ext_tool_path, file_path.display());

                    match Command::new(&ext_tool_path)
                        .arg(&file_path)
                        .spawn()
                    {
                        Ok(_) => {
                            tracing::info!("Successfully launched external tool for {}", file_name);
                        }
                        Err(e) => {
                            tracing::error!("Failed to launch external tool: {}", e);
                            let error_msg = format!("Failed to open BA2 file:\n{}", e);
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = weak_clone.upgrade() {
                                    show_toast(&ui, ToastData {
                                        message: error_msg,
                                        notification_type: NotificationType::Error,
                                        show: true,
                                    });
                                }
                            });
                        }
                    }
                });
            }
            _ => {
                tracing::warn!("Unknown file action: {}", action_str);
            }
        }
    });
}

/// Set up open extraction folder callback (Phase 2.3)
fn setup_open_folder_callback(main_window: &MainWindow, state: Arc<Mutex<AppState>>) {
    let weak = main_window.as_weak();

    main_window.on_open_extraction_folder(move || {
        let extraction_path = if let Some(ui) = weak.upgrade() {
            ui.get_extraction_folder().to_string()
        } else {
            return;
        };

        if extraction_path.is_empty() {
            // Fallback to config extraction path or current directory
            let app_state = state.lock().unwrap();
            let default_path = if app_state.config.advanced.extraction_path.is_empty() {
                std::env::current_dir()
                    .ok()
                    .and_then(|p| p.to_str().map(String::from))
                    .unwrap_or_else(|| ".".to_string())
            } else {
                app_state.config.advanced.extraction_path.clone()
            };

            tracing::info!("Opening extraction folder (default): {}", default_path);

            if let Err(e) = open::that(&default_path) {
                tracing::error!("Failed to open folder: {}", e);
                let error_msg = format!("Failed to open folder:\n{}", e);
                let weak_clone = weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak_clone.upgrade() {
                        show_toast(&ui, ToastData {
                            message: error_msg,
                            notification_type: NotificationType::Error,
                            show: true,
                        });
                    }
                });
            }
        } else {
            tracing::info!("Opening extraction folder: {}", extraction_path);

            if let Err(e) = open::that(&extraction_path) {
                tracing::error!("Failed to open folder: {}", e);
                let error_msg = format!("Failed to open folder:\n{}", e);
                let weak_clone = weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = weak_clone.upgrade() {
                        show_toast(&ui, ToastData {
                            message: error_msg,
                            notification_type: NotificationType::Error,
                            show: true,
                        });
                    }
                });
            }
        }
    });
}

/// Refresh the file table with optional threshold filtering (Phase 2.3)
fn refresh_file_table(ui: &MainWindow, state: &Arc<Mutex<AppState>>, threshold: Option<u64>) {
    let app_state = state.lock().unwrap();
    let entries = app_state.file_entries.entries();

    // Filter by threshold if provided
    let filtered_entries: Vec<&FileEntry> = if let Some(threshold_bytes) = threshold {
        entries.iter()
            .filter(|e| e.file_size <= threshold_bytes)
            .collect()
    } else {
        entries.iter().collect()
    };

    let row_data: Vec<FileRowData> = filtered_entries
        .iter()
        .map(|e| FileRowData {
            file_name: SharedString::from(&e.file_name),
            file_size: SharedString::from(e.size_display()),
            num_files: SharedString::from(e.file_count_display()),
            mod_name: SharedString::from(e.mod_display()),
            is_bad: e.is_corrupted(),
        })
        .collect();

    let total_size: u64 = filtered_entries.iter().map(|e| e.file_size).sum();

    ui.set_file_list(ModelRc::new(VecModel::from(row_data)));
    ui.set_total_files(filtered_entries.len() as i32);
    ui.set_total_size(SharedString::from(format_size(total_size, BINARY)));

    tracing::debug!(
        "Refreshed table: {} files shown{}",
        filtered_entries.len(),
        if threshold.is_some() { " (filtered)" } else { "" }
    );
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
