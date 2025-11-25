# Unpackrr (Rust Port)

## Project Overview
**Unpackrr** is a high-performance, graphical application for batch unpacking Bethesda Archive 2 (BA2) files, primarily for Fallout 4 and Fallout 76. It is a Rust port of an original Python application, designed for better performance, type safety, and a modern Fluent Design UI using the [Slint](https://slint.dev/) framework.

### Key Technologies
*   **Language**: Rust (2024 Edition)
*   **GUI Framework**: [Slint](https://slint.dev/) (`.slint` files in `ui/`)
*   **Async Runtime**: `tokio` (bridged to Slint via `async-compat`)
*   **Parallel Processing**: `rayon`
*   **BA2 Extraction**: Delegates to `BSArch.exe` (external tool)/
*   **Logging**: `tracing`

## Build & Run Instructions

### Prerequisites
*   Rust (latest stable, 2024 edition support)
*   `BSArch.exe` (must be placed in the same directory as the executable or configured in settings)

### Commands
*   **Build (Dev)**: `cargo build`
*   **Build (Release)**: `cargo build --release`
*   **Run**: `cargo run`
*   **Test**: `cargo test`
    *   *Note*: Some tests may require `BSArch.exe` or sample BA2 files.
*   **Format**: `cargo fmt`
*   **Lint**: `cargo clippy`

## Project Structure

*   `src/`: Rust source code.
    *   `main.rs`: Application entry point.
    *   `lib.rs`: Library root.
    *   `ba2/`: BA2 format handling and parsing logic.
    *   `config/`: Configuration management (serialization/deserialization).
    *   `ui/`: Rust-side UI logic and callbacks.
    *   `operations/`: Core business logic (extraction, scanning).
*   `ui/`: Slint UI definitions.
    *   `main.slint`: Main window layout and styling.
*   `tests/`: Integration tests.

## Development Conventions

### Rust & Slint Integration
*   **Strict Separation**: Keep UI layout in `.slint` files and logic in Rust.
*   **Async Bridge**: Use `async-compat` to run Tokio futures within the Slint event loop.
    *   *Pattern*: Use `slint::spawn_local(async_compat::Compat::new(async { ... }))` for main-thread async tasks.
    *   *Pattern*: Use `slint::invoke_from_event_loop` to update UI from background threads.
*   **State Management**: Use `Rc<Vec<FileItem>>` (Slint models) for list data.

### Coding Style
*   **Error Handling**: Use `anyhow` for app errors and `thiserror` for libraries.
*   **Path Handling**: Always use `PathBuf` and `dunce` for Windows path compatibility.
*   **Logging**: Use `tracing` for all log output.

### Testing
*   Write unit tests within `src/` modules (`#[cfg(test)]`).
*   Write integration tests in `tests/`.
*   Mock file system operations where possible to avoid dependency on real BA2 files for unit tests.

## AI Agent Instructions
*   **Context**: This is a Windows-first application. Ensure path handling is Windows-compatible.
*   **UI Changes**: modifications to `ui/*.slint` often require corresponding updates to `src/ui/*.rs` or `src/main.rs` to handle callbacks and properties.
*   **External Tools**: The app relies on `BSArch.exe` for the heavy lifting of extraction. Do not attempt to re-implement full BA2 extraction logic in pure Rust unless specifically asked; focus on the orchestration and UI.
