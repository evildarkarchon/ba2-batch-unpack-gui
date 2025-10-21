# Unpackrr-rs Development Guide

This project is a Rust port of the Python-based BA2 batch unpacker GUI. The goal is to leverage Rust's performance, safety, and modern tooling while maintaining feature parity with the original application.

## Project Context

**Original**: Python GUI application using PyQt/PySide for extracting Bethesda Archive 2 (BA2) files
**Target**: High-performance Rust implementation in [unpackrr-rs/](unpackrr-rs/)
**Edition**: Rust 2024 (latest)

## Rust Best Practices (Edition 2024)

### Language Features

- **Prefer `let-else` patterns** for early returns instead of nested matches
- **Use `if let` chains** for multiple pattern matching when appropriate
- **Leverage `impl Trait`** in return positions for cleaner APIs
- **Use `async`/`await`** for I/O operations, especially file operations
- **Prefer `?` operator** over explicit error handling where possible
- **Use pattern matching exhaustively** - avoid catch-all patterns unless necessary

### Error Handling

- **Use `anyhow`** for application-level errors with context
- **Use `thiserror`** for library-level custom error types
- **Always add context** to errors using `.context()` or `.with_context()`
- **Never use `.unwrap()`** in production code - handle all errors explicitly
- **Avoid `.expect()`** unless truly impossible to fail (document why)

```rust
// Good
let file = File::open(path)
    .with_context(|| format!("Failed to open BA2 file: {}", path.display()))?;

// Bad
let file = File::open(path).unwrap();
```

### Memory and Performance

- **Prefer `&str` over `String`** in function parameters when you don't need ownership
- **Use `Cow<str>`** when you might need to modify strings conditionally
- **Use iterators** instead of collecting into intermediate vectors
- **Leverage zero-cost abstractions** - don't avoid abstractions for "performance" without profiling
- **Use `#[inline]`** judiciously for hot paths (only after profiling)
- **Prefer stack allocation** over heap when size is known and reasonable

### Code Organization

- **Module structure should mirror conceptual domains** (not Python file structure)
- **Use `mod.rs` or `lib.rs`** to expose public APIs clearly
- **Prefer private by default** - only expose what's necessary
- **Group related functionality** in modules
- **Use workspace features** if the project grows to multiple crates

### Async Patterns

- **Use `tokio` runtime** for async operations (file I/O, networking)
- **Prefer `async fn` over `-> impl Future`** for clarity
- **Use `async_trait`** macro for trait methods that need to be async
- **Avoid blocking operations** in async contexts - use `spawn_blocking`
- **Use channels** (`tokio::sync::mpsc`) for communication between async tasks

### Testing

- **Write unit tests** in the same file as the code (`#[cfg(test)]` module)
- **Write integration tests** in `tests/` directory
- **Use `#[test]`** for synchronous tests
- **Use `#[tokio::test]`** for async tests
- **Mock external dependencies** (file system, network) in tests
- **Use property-based testing** (`proptest`) for complex algorithms
- **Benchmark performance-critical code** using `criterion`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ba2_header_parsing() {
        // Test code
    }

    #[tokio::test]
    async fn test_async_extraction() {
        // Async test code
    }
}
```

## Project-Specific Guidelines

### BA2 File Format

- **BA2 files are Bethesda archives** used in Fallout 4 and Fallout 76
- **Support multiple BA2 formats**: General, Texture (DX10, BC1-7)
- **Handle corruption gracefully** - many mods have corrupted archives
- **Validate headers and checksums** before extraction
- **Support both listing and extraction** operations

### Porting from Python

#### Do NOT directly translate Python idioms:
- **Avoid Python-style exception handling** - use Result types
- **Don't use panics for control flow** - Python's exceptions != Rust panics
- **Avoid deeply nested structures** - flatten where reasonable
- **Don't port Python's dynamic typing** - leverage Rust's type system

#### Study existing Rust BA2 libraries:
- Check crates.io for BA2/BSA handling crates
- Look at `bsa-rs`, `ba2`, or similar crates
- **Copy successful patterns** rather than reinventing

#### Performance considerations:
- **Python code may be I/O bound** - Rust can be CPU bound if not careful
- **Use memory mapping** (`memmap2`) for reading large BA2 files
- **Parallelize extraction** using `rayon` for multiple files
- **Stream large files** instead of loading entirely into memory

### GUI Framework

This project uses **Slint** for the GUI layer with **Fluent Design** styling:

#### Slint Best Practices
- **Separate UI from logic** - keep `.slint` files focused on presentation
- **Use callbacks** to communicate from Slint to Rust backend
- **Use properties** for data binding from Rust to UI
- **Leverage Slint's built-in components** before creating custom ones
- **Use `@tr()` macro** for internationalization (maintaining multi-language support from Python version)
- **Structure `.slint` files hierarchically** - create reusable components

#### Fluent Design Implementation
- **Follow Microsoft Fluent Design principles** as closely as Slint allows:
  - Light and depth (subtle shadows, layering)
  - Motion (smooth transitions and animations)
  - Material (translucent/acrylic effects where possible)
  - Scale (responsive layout for different window sizes)
- **Use Fluent color palette** - maintain the Python version's theme support (light/dark mode)
- **Rounded corners** on buttons and cards (Fluent standard)
- **Accent colors** for interactive elements
- **Typography hierarchy** - clear visual hierarchy matching Fluent guidelines

#### Slint-Rust Communication
```rust
// Good pattern: Define callbacks in .slint, implement in Rust
slint::slint! {
    export component MainWindow {
        callback start-extraction(string);
        callback cancel-operation();
    }
}

let ui = MainWindow::new()?;
ui.on_start_extraction(|path| {
    // Handle extraction logic
});
```

#### State Management
- **Use Slint properties** for UI state (progress, status text, etc.)
- **Use Rust channels** to communicate between async backend and UI thread
- **Update UI from main thread only** - use `slint::invoke_from_event_loop()`
- **Avoid blocking the UI thread** - run extraction in background tasks

#### Theme Support
- **Implement light/dark themes** using Slint's theming system
- **Allow custom accent colors** matching the Python version's personalization
- **Store theme preferences** in configuration file
- **Apply themes at runtime** without restart

#### Slint + Tokio Integration (CRITICAL)

**⚠️ Event Loop Conflict Warning**: Slint and Tokio have separate event loops that WILL conflict if not handled correctly. Follow these patterns strictly:

**DO NOT:**
- ❌ Use `#[tokio::main]` - this creates a current-thread runtime that conflicts with Slint
- ❌ Use Tokio's current-thread runtime on the main thread - Slint can't yield to it
- ❌ Block the Slint UI thread with `.await` or blocking operations
- ❌ Try to run `tokio::spawn` directly from UI callbacks without proper wrapping

**DO:**
- ✅ Use `async-compat` crate to bridge Tokio futures into Slint's executor
- ✅ Use `slint::spawn_local()` for all main thread async tasks
- ✅ Use `slint::invoke_from_event_loop()` to communicate from background threads to UI
- ✅ Run heavy async work on a separate multi-threaded Tokio runtime
- ✅ Use `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for shared state between UI and async tasks

**Recommended Pattern:**

```rust
use slint::spawn_local;
use async_compat::Compat;
use tokio::sync::mpsc;
use std::sync::Arc;

// Initialize channels and state BEFORE setting up callbacks
let (tx, mut rx) = mpsc::channel(100);
let shared_state = Arc::new(Mutex::new(AppState::default()));

// For main thread async work (UI updates, light tasks)
ui.on_some_action(move || {
    let tx = tx.clone();
    spawn_local(Compat::new(async move {
        // Tokio future running in Slint's event loop
        let result = some_async_operation().await;
        tx.send(result).await.unwrap();
    })).unwrap();
});

// For heavy background work (file extraction, etc.)
std::thread::spawn(move || {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        while let Some(msg) = rx.recv().await {
            // Heavy processing here
            let result = process_ba2_file(msg).await;

            // Update UI from background thread
            let ui_weak = ui_weak.clone();
            slint::invoke_from_event_loop(move || {
                let ui = ui_weak.upgrade().unwrap();
                ui.set_progress(result.progress);
            }).unwrap();
        }
    });
});
```

**Key Points:**
- **async-compat::Compat::new()** wraps Tokio futures for Slint compatibility
- **spawn_local()** executes futures on Slint's main thread executor
- **invoke_from_event_loop()** safely calls UI updates from background threads
- **Initialize before callbacks** - avoid moving values into closures that need multiple access
- **Separate runtimes** - use a dedicated Tokio runtime on background threads for heavy work

### File Paths

- **Use `std::path::PathBuf` and `Path`** - never use strings for paths
- **Handle Windows paths correctly** - this is Windows-primary software
- **Use `dunce`** crate to handle UNC paths on Windows
- **Canonicalize paths** before comparisons
- **Check path existence** before operations

### Configuration and Settings

- **Use `serde`** for serialization/deserialization
- **Support TOML** for config files (`toml` crate)
- **Validate configuration** on load with clear error messages
- **Provide sensible defaults** matching the Python version
- **Store config in standard locations** (`directories` crate)

### Regex Patterns

The Python version supports regex in ignored files:
- **Use `regex` crate** (already well-optimized)
- **Compile regex patterns once** and cache them
- **Validate regex at config load time** - not at match time
- **Provide clear error messages** for invalid patterns

## Code Quality Standards

### Before Committing

- **Run `cargo fmt`** to format code
- **Run `cargo clippy`** and fix all warnings
- **Run `cargo test`** and ensure all tests pass
- **Run `cargo build --release`** to verify release builds work
- **Check `cargo doc --no-deps --open`** for documentation quality

### Clippy Configuration

Add to `Cargo.toml`:
```toml
[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
# Allow some pedantic lints that conflict with readability
must_use_candidate = "allow"
missing_errors_doc = "allow"
```

### Documentation

- **Document all public APIs** with `///` doc comments
- **Include examples** in doc comments where helpful
- **Explain non-obvious design decisions** in code comments
- **Keep comments up to date** when code changes
- **Use `//!` for module-level documentation**

### Commit Messages

- **Follow conventional commits** format:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `refactor:` for code refactoring
  - `test:` for test additions/changes
  - `docs:` for documentation
  - `chore:` for maintenance tasks

## Common Pitfalls to Avoid

### Windows-Specific Issues

- **Never redirect to `nul`** in commands - it creates an undeletable file
- **Use `std::process::Command` correctly** for spawning processes
- **Handle case-insensitive paths** where needed (Windows filesystems)
- **Test with paths containing spaces** and special characters

### Memory Management

- **Don't clone unnecessarily** - use references when possible
- **Watch for cycles** in `Rc`/`Arc` - use `Weak` to break cycles
- **Profile memory usage** for large BA2 files (use `heaptrack` or similar)
- **Stream processing** for files larger than available RAM

### Error Recovery

- **Fail gracefully** - don't crash on corrupted BA2 files
- **Provide actionable error messages** to users
- **Log errors with context** using `tracing` or `log` crate
- **Implement retry logic** for transient file system errors

### Cross-Platform Compatibility

While Windows is primary:
- **Use `std::path` instead of string manipulation** for paths
- **Use `std::env::consts::OS`** to detect platform when needed
- **Avoid Windows-only APIs** unless necessary
- **Test on both debug and release builds** (behavior can differ)

## Development Workflow

1. **Understand Python functionality** before porting
2. **Write tests first** for the functionality
3. **Implement minimal working version**
4. **Iterate on performance** if needed
5. **Document public APIs**
6. **Review and refactor** before moving to next feature

## Dependency Management

### Recommended Crates

- **GUI Framework**: `slint` (with `slint-build` in build-dependencies)
- **Async compatibility**: `async-compat` (REQUIRED for Slint + Tokio integration)
- **Error handling**: `anyhow`, `thiserror`
- **Async runtime**: `tokio` (for backend file operations)
- **Serialization**: `serde`, `serde_json`, `toml`
- **Logging**: `tracing`, `tracing-subscriber`
- **Path handling**: `dunce`, `directories`
- **Regex**: `regex`
- **Parallel processing**: `rayon`
- **Memory mapping**: `memmap2`
- **Internationalization**: Built into Slint via `@tr()` macro

### Version Pinning

- **Use `cargo update`** regularly to keep dependencies fresh
- **Review `cargo outdated`** periodically
- **Pin major versions** in `Cargo.toml` using `^` (default)
- **Test thoroughly** after dependency updates

## Resources

### Rust
- [Rust Book (Edition 2024)](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Effective Rust](https://www.lurklurk.org/effective-rust/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

### Slint GUI
- [Slint Documentation](https://slint.dev/docs/)
- [Slint Rust API Reference](https://slint.dev/docs/rust/slint/)
- [Slint Examples](https://github.com/slint-ui/slint/tree/master/examples)
- [Slint Language Reference](https://slint.dev/docs/slint/)

### Fluent Design
- [Microsoft Fluent 2 Design System](https://fluent2.microsoft.design/)
- [Fluent UI Design Principles](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/design-principles)
- [Fluent Color Palette](https://fluent2.microsoft.design/color)

## Notes for AI Assistants

- **Always check for existing BA2-handling crates** before implementing from scratch
- **Study the Python code** in `src/` to understand business logic
- **Preserve feature parity** with Python version unless explicitly changing
- **Use Slint for all GUI code** - UI should be in `.slint` files, logic in Rust
- **Follow Fluent Design aesthetics** - rounded corners, subtle shadows, smooth animations
- **Maintain theme support** - light/dark mode and customizable accent colors are required features
- **Keep UI responsive** - never block the main thread with file operations
- **Optimize for developer experience** first, performance second (profile before optimizing)
- **Ask for clarification** if Python code behavior is ambiguous
- **Test with real BA2 files** from Nexus Mods when possible
