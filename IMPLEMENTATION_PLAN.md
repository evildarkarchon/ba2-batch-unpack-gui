# Unpackrr-rs Implementation Plan

**Port Status**: Planning Phase
**Target**: Complete Rust port with feature parity to Python version
**Framework**: Slint + Fluent Design
**Rust Edition**: 2024

---

## Project Overview

This document outlines the phased implementation plan for porting the Python-based BA2 Batch Unpacker GUI to Rust. This is a **GUI wrapper** around the excellent BSArch.exe command-line tool, focusing on providing an intuitive interface rather than reimplementing extraction logic.

The port aims to achieve:

- ✅ Complete feature parity with Python version
- ✅ Improved performance and reliability through Rust
- ✅ Enhanced type safety and error handling
- ✅ Modern async architecture with Tokio
- ✅ Fluent Design UI via Slint framework
- ✅ Portable deployment (no installer required)

**Original Project**: [ba2-batch-unpack-gui](https://github.com/kazum1kun/ba2-batch-unpack-gui) by KazumaKuun
**Current Maintainer**: [evildarkarchon](https://github.com/evildarkarchon)

**Philosophy**: Don't reinvent the wheel - BSArch.exe is years ahead of anything we could implement. Focus on user experience.

**BSArch.exe**: Licensed under MPL-2.0 (Mozilla Public License 2.0), which permits redistribution. We bundle it with proper attribution.

---

## Phase 1: Foundation & Infrastructure (MVP)

**Goal**: Establish core architecture and basic functionality

### 1.1 Project Setup & Dependencies

**Priority**: Critical
**Estimated Effort**: 1-2 days

- [x] Initialize Rust project structure in `unpackrr-rs/`
- [ ] Configure `Cargo.toml` with essential dependencies:
  - `slint` (GUI framework)
  - `async-compat` (Slint+Tokio bridge)
  - `tokio` (async runtime)
  - `anyhow` & `thiserror` (error handling)
  - `serde`, `serde_json` (serialization)
  - `regex` (pattern matching)
  - `directories` (config paths)
  - `tracing`, `tracing-subscriber` (logging)
- [ ] Set up build configuration for Slint
- [ ] Configure Clippy lints (as per CLAUDE.md)
- [ ] Create initial module structure:
  ```
  src/
    ├── main.rs
    ├── lib.rs
    ├── error.rs
    ├── config/
    ├── ba2/
    ├── operations/
    └── ui/
  ```

**Deliverables**:
- Compiling Rust project skeleton
- CI-ready configuration (fmt, clippy, test)

---

### 1.2 Error Handling Foundation

**Priority**: Critical
**Estimated Effort**: 1 day

- [ ] Create `src/error.rs` with custom error types
- [ ] Define error categories:
  - `ConfigError` - Configuration issues
  - `BA2Error` - BA2 format/parsing errors
  - `IOError` - File system operations
  - `ValidationError` - Input validation
- [ ] Implement `thiserror` derives for custom errors
- [ ] Use `anyhow::Context` for error propagation
- [ ] Add error display helpers for user-facing messages

**Deliverables**:
- Comprehensive error type system
- Clear error messages for end users

---

### 1.3 Configuration Management

**Priority**: Critical
**Estimated Effort**: 2-3 days

**Files to Port**: `src/misc/Config.py`

- [ ] Create `src/config/mod.rs` structure
- [ ] Define configuration structs with `serde` derives:
  ```rust
  struct AppConfig {
      extraction: ExtractionConfig,
      appearance: AppearanceConfig,
      advanced: AdvancedConfig,
      update: UpdateConfig,
  }
  ```
- [ ] Implement configuration sections:
  - `ExtractionConfig`: postfixes, ignored files, auto backup, etc.
  - `AppearanceConfig`: language, theme mode, theme color
  - `AdvancedConfig`: debug mode, paths, external tools
  - `UpdateConfig`: auto-check settings
- [ ] Configuration file handling:
  - [ ] Default config generation
  - [ ] Load from `config/config.json`
  - [ ] Save on changes
  - [ ] Validation on load
- [ ] Regex pattern compilation and caching
- [ ] Path resolution (relative/absolute, Windows-safe)
- [ ] Unit tests for:
  - Default config generation
  - Serialization/deserialization
  - Path validation
  - Regex compilation

**Deliverables**:
- Complete configuration management system
- Validated JSON persistence
- Pre-compiled regex patterns

---

### 1.4 BA2 File Format Support

**Priority**: Critical
**Estimated Effort**: 3-5 days

**Files to Port**: `src/misc/Utilities.py` (BA2-related functions)

#### 1.4.1 Header Parsing

- [ ] Create `src/ba2/header.rs`
- [ ] Define BA2 header struct:
  ```rust
  struct BA2Header {
      magic: [u8; 4],      // "BTDX"
      version: u32,
      archive_type: String, // "GNRL" or texture types
      file_count: u32,
      names_offset: u64,
  }
  ```
- [ ] Implement binary parsing (consider `binrw` or `nom` crate)
- [ ] Header validation logic
- [ ] Magic number verification

#### 1.4.2 BA2 Utilities

- [ ] Create `src/ba2/parser.rs`
- [ ] Port `num_files_in_ba2()` function
- [ ] Implement BA2 validation without extraction
- [ ] Support different BA2 types (General, DX10, BC1-7)

#### 1.4.3 BSArch.exe Integration

- [ ] Create `src/ba2/extractor.rs`
- [ ] Implement `BSArch.exe` wrapper for extraction:
  - [ ] Command building
  - [ ] Process spawning
  - [ ] Output parsing
  - [ ] Error handling
- [ ] Bundle `BSArch.exe` in resources
- [ ] Cross-platform path handling
- [ ] Include BSArch.exe license file (MPL-2.0) in distribution

**Note**: BSArch.exe is the extraction engine - we're building a GUI around it, not replacing it. Licensed under MPL-2.0.

#### 1.4.4 Testing

- [ ] Unit tests for header parsing
- [ ] Integration tests with sample BA2 files
- [ ] Error handling tests (corrupted files)

**Deliverables**:
- BA2 file validation
- External extraction via BSArch.exe
- Robust error handling for corrupted archives

---

### 1.5 File System Operations

**Priority**: High
**Estimated Effort**: 2-3 days

**Files to Port**: Parts of `src/misc/Utilities.py`

#### 1.5.1 Core Utilities

- [ ] Create `src/operations/scan.rs`
- [ ] Port `scan_for_ba2()` logic:
  - [ ] Directory traversal (second-tier folders)
  - [ ] BA2 file discovery
  - [ ] Postfix filtering
  - [ ] Ignored pattern matching (exact, substring, regex)
- [ ] Parallel scanning with `rayon`
- [ ] Progress reporting via channels

#### 1.5.2 Size Parsing

- [ ] Port `parse_size()` function
- [ ] Support units: B, KB, MB, GB, TB
- [ ] Case-insensitive parsing
- [ ] Use `humansize` crate for formatting

#### 1.5.3 Path Handling

- [ ] Windows UNC path support (`dunce` crate)
- [ ] Path canonicalization
- [ ] Relative/absolute path resolution
- [ ] Case-insensitive path comparison

#### 1.5.4 Testing

- [ ] Unit tests for size parsing
- [ ] Integration tests for directory scanning
- [ ] Path handling edge cases (spaces, special chars, UNC paths)

**Deliverables**:
- Async directory scanning
- Robust path handling
- Pattern matching (exact/substring/regex)

---

### 1.6 Data Models

**Priority**: High
**Estimated Effort**: 1 day

**Files to Port**: `src/model/PreviewTableModel.py`

- [ ] Create `src/models/mod.rs`
- [ ] Define `FileEntry` struct:
  ```rust
  struct FileEntry {
      file_name: String,
      file_size: u64,
      num_files: u32,
      dir_name: String,
      full_path: PathBuf,
      is_bad: bool,
  }
  ```
- [ ] Implement sorting logic
- [ ] Humanized size display
- [ ] Add `Debug`, `Clone`, `PartialEq` derives

**Deliverables**:
- Type-safe data models
- Sortable file entries

---

### 1.7 Basic Slint UI Setup

**Priority**: High
**Estimated Effort**: 3-4 days

**Files to Port**: Initial structure from `src/MainWindow.py`

#### 1.7.1 Main Window Shell

- [ ] Create `ui/main.slint`
- [ ] Fluent Design base styling:
  - [ ] Color palette (light/dark themes)
  - [ ] Typography hierarchy
  - [ ] Rounded corners (8px standard)
  - [ ] Shadow system
- [ ] Window configuration:
  - [ ] Minimum size: 1000x700px
  - [ ] Icon setup
  - [ ] Title bar

#### 1.7.2 Navigation Sidebar

- [ ] Icon-based navigation
- [ ] Three sections:
  - [ ] Extraction screen
  - [ ] Check Files screen
  - [ ] Settings screen
- [ ] Active state highlighting
- [ ] Fluent Design sidebar styling

#### 1.7.3 Slint-Rust Integration

- [ ] Create `src/ui/mod.rs`
- [ ] Set up Slint component callbacks
- [ ] Establish UI state management
- [ ] Configure `async-compat` for Tokio integration
- [ ] Implement `spawn_local()` pattern for UI updates

**Deliverables**:
- Functional main window shell
- Navigation system
- Slint+Tokio event loop integration

---

### 1.8 Extraction Screen (MVP)

**Priority**: Critical
**Estimated Effort**: 4-5 days

**Files to Port**: `src/view/MainScreen.py` (simplified version)

#### 1.8.1 UI Components

- [ ] Create `ui/screens/extraction.slint`
- [ ] Folder selection:
  - [ ] Folder picker button
  - [ ] Path text field
  - [ ] Drag-and-drop support (Phase 2)
- [ ] File preview table:
  - [ ] Columns: Filename, Size, File Count, Mod Folder
  - [ ] Sortable headers
  - [ ] Data binding from Rust
- [ ] Action buttons:
  - [ ] Start extraction button
  - [ ] Cancel button (Phase 2)
- [ ] Status display:
  - [ ] Total files count
  - [ ] Total size
  - [ ] Progress bar (Phase 2)

#### 1.8.2 Backend Logic

- [ ] Create `src/operations/extract.rs`
- [ ] Folder selection handling
- [ ] BA2 scanning on folder change
- [ ] Table population from scan results
- [ ] Extraction orchestration:
  - [ ] Call BSArch.exe for each file
  - [ ] File backup logic
  - [ ] File cleanup (delete/move)
  - [ ] Failed file tracking
- [ ] Progress reporting via channels
- [ ] UI updates via `invoke_from_event_loop()`

#### 1.8.3 Integration

- [ ] Wire UI callbacks to backend
- [ ] State management for extraction process
- [ ] Error display to user
- [ ] Success notifications

**Deliverables**:
- Functional extraction screen
- End-to-end BA2 extraction workflow
- Progress feedback to user

---

### 1.9 Logging System

**Priority**: Medium
**Estimated Effort**: 1-2 days

**Files to Port**: `src/view/LogView.py`

- [ ] Configure `tracing-subscriber`
- [ ] Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- [ ] Console output formatting
- [ ] File logging (optional)
- [ ] Debug log view (Phase 2 UI)

**Deliverables**:
- Comprehensive logging
- Debug diagnostics capability

---

### Phase 1 Success Criteria

- ✅ Application compiles and runs
- ✅ Configuration loads/saves correctly
- ✅ Folder selection works
- ✅ BA2 files are scanned and displayed
- ✅ Extraction process completes successfully
- ✅ Backup/cleanup logic functions
- ✅ Basic error handling in place

**Estimated Total Time**: 2-3 weeks

---

## Phase 2: Feature Completeness

**Goal**: Achieve full feature parity with Python version

### 2.1 File Validation Screen

**Priority**: High
**Estimated Effort**: 2-3 days

**Files to Port**: `src/view/CheckFileScreen.py`, `src/misc/BsaChecker.py`

- [ ] Create `ui/screens/validation.slint`
- [ ] UI components:
  - [ ] Folder selector
  - [ ] Deep scan checkbox
  - [ ] Results text area
  - [ ] Start/Cancel buttons
- [ ] Create `src/operations/validate.rs`
- [ ] Implement validation modes:
  - [ ] Quick scan (list BA2 contents)
  - [ ] Deep scan (extract to temp, verify, cleanup)
- [ ] Progress reporting
- [ ] Issue tracking and display

**Deliverables**:
- Functional file validation screen
- Quick and deep scan modes
- Detailed issue reporting

---

### 2.2 Settings Screen

**Priority**: High
**Estimated Effort**: 3-4 days

**Files to Port**: `src/view/SettingScreen.py`

#### 2.2.1 UI Layout

- [ ] Create `ui/screens/settings.slint`
- [ ] Setting sections:
  - [ ] Extraction settings
  - [ ] Personalization
  - [ ] Update settings
  - [ ] Advanced settings
  - [ ] About section

#### 2.2.2 Extraction Settings

- [ ] Postfixes input (comma-separated)
- [ ] Ignored files input with regex indicator
- [ ] Ignore bad files toggle
- [ ] Auto backup toggle

#### 2.2.3 Personalization

- [ ] Theme mode picker (Light/Dark/System)
- [ ] Accent color picker
- [ ] Language selector (Auto/EN/中文简体/中文繁體)

#### 2.2.4 Update Settings

- [ ] Check updates at startup toggle

#### 2.2.5 Advanced Settings

- [ ] Show debug log toggle
- [ ] Extraction path input
- [ ] Backup path input
- [ ] External BA2 tool file picker

#### 2.2.6 About Section

**Credits Structure**:
- [ ] Original author section:
  - Author: KazumaKuun / Southwest Codeworks
  - Links: GitHub, Nexus Mods, Ko-fi
- [ ] Current maintainer section:
  - Maintainer: evildarkarchon
  - Links: GitHub
- [ ] Version information
- [ ] License information:
  - Application: GPL-3.0
  - BSArch.exe: MPL-2.0 (bundled third-party tool)
- [ ] BSArch.exe attribution and link
- [ ] Year: 2024

#### 2.2.7 Settings Persistence

- [ ] Real-time config updates
- [ ] Validation on input
- [ ] Config save on change
- [ ] Restart required indicator (for language changes)

**Deliverables**:
- Complete settings screen
- Live configuration updates
- All personalization options working
- Proper attribution for original author and current maintainer

---

### 2.3 Enhanced Extraction Features

**Priority**: High
**Estimated Effort**: 2-3 days

#### 2.3.1 Size Threshold

- [ ] Auto threshold calculation (235-file limit logic)
- [ ] Manual threshold input
- [ ] "Auto" toggle button
- [ ] Real-time table filtering

#### 2.3.2 Table Enhancements

- [ ] Context menu (Ignore file, Open externally)
- [ ] Double-click to open file
- [ ] Bad file highlighting (dark red background)
- [ ] Row hiding for extracted files

#### 2.3.3 Drag-and-Drop

- [ ] Folder drag-and-drop support
- [ ] Visual feedback on drag hover

#### 2.3.4 Progress Tracking

- [ ] Extraction progress bar
- [ ] Current file indicator
- [ ] Estimated time remaining
- [ ] Cancellation support

**Deliverables**:
- Advanced table functionality
- Size threshold filtering
- Real-time progress feedback
- Cancellable operations

---

### 2.4 Theme System

**Priority**: Medium
**Estimated Effort**: 2-3 days

#### 2.4.1 Theme Implementation

- [ ] Light theme color palette
- [ ] Dark theme color palette
- [ ] System theme detection (Windows)
- [ ] Dynamic theme switching (no restart)

#### 2.4.2 Custom Accent Colors

- [ ] Color picker integration
- [ ] Accent color application across UI
- [ ] Fluent Design compliant highlighting

#### 2.4.3 Theme Persistence

- [ ] Save theme preference
- [ ] Apply on startup

**Deliverables**:
- Light/dark/system themes
- Custom accent colors
- Persistent theme preferences

---

### 2.5 Internationalization

**Priority**: Medium
**Estimated Effort**: 2-3 days

#### 2.5.1 Translation Infrastructure

- [ ] Set up Slint `@tr()` macro usage
- [ ] Create translation files:
  - [ ] English (en)
  - [ ] Simplified Chinese (zh-CN)
  - [ ] Traditional Chinese (zh-TW)
- [ ] Auto-detect system language

#### 2.5.2 String Extraction

- [ ] Extract all UI strings
- [ ] Mark for translation with `@tr()`
- [ ] Translate to Chinese variants (may need assistance)

#### 2.5.3 Language Switching

- [ ] Language selector in settings
- [ ] Apply language on restart
- [ ] Language persistence

**Deliverables**:
- Multi-language support (EN, 中文)
- Language switching functionality

---

### 2.6 Update Checker

**Priority**: Medium
**Estimated Effort**: 1-2 days

**Files to Port**: `src/misc/Utilities.py` (`check_latest_version()`)

- [ ] GitHub API integration (`reqwest` crate)
- [ ] Version comparison (`semver` crate)
- [ ] Update notification dialog
- [ ] Optional auto-check at startup
- [ ] Manual check from settings
- [ ] Check against current maintainer's fork repository

**Deliverables**:
- Update checking functionality
- User notifications for new versions

---

### 2.7 Notifications & Dialogs

**Priority**: Medium
**Estimated Effort**: 2 days

**Files to Port**: `src/prefab/InfoBar.py`, `src/prefab/MessageBox.py`

- [ ] Create Slint notification components
- [ ] Toast notifications (InfoBar equivalent):
  - [ ] Success messages
  - [ ] Error messages
  - [ ] Warning messages
  - [ ] Info messages
- [ ] Modal dialogs (MessageBox equivalent):
  - [ ] Confirmation dialogs
  - [ ] Error dialogs
  - [ ] Custom message dialogs
- [ ] Fluent Design styling for notifications

**Deliverables**:
- Toast notification system
- Modal dialog components
- Consistent messaging UX

---

### 2.8 Advanced Error Handling

**Priority**: High
**Estimated Effort**: 2 days

- [ ] User-friendly error messages
- [ ] Error recovery suggestions
- [ ] Retry logic for transient failures
- [ ] Detailed error logging
- [ ] Error reporting UI (copy error details)

**Deliverables**:
- Comprehensive error handling
- Helpful user guidance
- Debugging capabilities

---

### 2.9 Windows Integration

**Priority**: Medium
**Estimated Effort**: 1-2 days

**Files to Port**: `src/misc/Utilities.py` (`get_default_windows_app()`)

- [ ] Windows registry access (`winreg` crate)
- [ ] Default `.ba2` file handler detection
- [ ] Auto-populate external tool setting
- [ ] File association querying

**Deliverables**:
- Windows registry integration
- Smart default tool detection

---

### Phase 2 Success Criteria

- ✅ All screens implemented and functional
- ✅ Full settings management
- ✅ Theme and language support
- ✅ Update checking works
- ✅ All Python features replicated
- ✅ Robust error handling throughout

**Estimated Total Time**: 3-4 weeks

---

## Phase 3: Polish & Distribution

**Goal**: Polish UI, comprehensive testing, and prepare for release

### 3.1 Advanced UI Polish

**Priority**: Medium
**Estimated Effort**: 1 week

#### 3.4.1 Animations

- [ ] Smooth screen transitions
- [ ] Button hover effects
- [ ] Progress bar animations
- [ ] Loading indicators
- [ ] Fluent motion principles

#### 3.4.2 Visual Effects

- [ ] Acrylic/translucency effects (if Slint supports)
- [ ] Drop shadows
- [ ] Layering and depth
- [ ] Hover states
- [ ] Focus indicators

#### 3.4.3 Responsiveness

- [ ] Window resizing handling
- [ ] Minimum size enforcement
- [ ] Adaptive layouts
- [ ] Font scaling

**Deliverables**:
- Polished, professional UI
- Smooth animations
- Excellent user experience

---

### 3.2 Splash Screen

**Priority**: Low
**Estimated Effort**: 1 day

**Files to Port**: `src/MainWindow.py` (splash screen logic)

- [ ] Slint splash screen component
- [ ] Application icon display
- [ ] Loading indicator
- [ ] Version display
- [ ] Timed display or loading-based

**Deliverables**:
- Professional startup experience

---

### 3.3 Debug Log View

**Priority**: Low
**Estimated Effort**: 1-2 days

**Files to Port**: `src/view/LogView.py`

- [ ] Create `ui/screens/log_view.slint`
- [ ] Log viewer component:
  - [ ] Color-coded log levels
  - [ ] Auto-scroll
  - [ ] Copy to clipboard
  - [ ] Clear log button
  - [ ] Filter by level
- [ ] Capture Rust panics
- [ ] Show/hide based on debug mode setting

**Deliverables**:
- Debug log window
- Developer diagnostics
- User troubleshooting aid

---

### 3.4 Testing Suite

**Priority**: High
**Estimated Effort**: 1 week

#### 3.7.1 Unit Tests

- [ ] Configuration management
- [ ] BA2 parsing
- [ ] File utilities
- [ ] Size parsing
- [ ] Pattern matching
- [ ] Path handling

#### 3.7.2 Integration Tests

- [ ] End-to-end extraction workflow
- [ ] Validation workflow
- [ ] Configuration persistence
- [ ] Theme switching
- [ ] Language switching

#### 3.7.3 UI Tests

- [ ] Slint component tests (if available)
- [ ] Manual testing checklist

#### 3.7.4 Performance Tests

- [ ] Benchmark extraction speed
- [ ] Benchmark scanning speed
- [ ] Memory usage profiling
- [ ] Large dataset testing (1000+ BA2s)

**Deliverables**:
- Comprehensive test coverage
- Performance benchmarks
- Regression prevention

---

### 3.5 Documentation

**Priority**: Medium
**Estimated Effort**: 3-5 days

#### 3.8.1 Code Documentation

- [ ] API documentation (`///` doc comments)
- [ ] Module documentation (`//!`)
- [ ] Examples in docs
- [ ] Generate docs with `cargo doc`

#### 3.8.2 User Documentation

- [ ] README.md update
- [ ] Installation guide
- [ ] Usage guide
- [ ] Configuration reference
- [ ] Troubleshooting guide
- [ ] Attribution section:
  - Original author (KazumaKuun)
  - Current maintainer (evildarkarchon)
  - BSArch.exe (MPL-2.0)
- [ ] License documentation (GPL-3.0 + third-party licenses)

#### 3.8.3 Developer Documentation

- [ ] Architecture overview
- [ ] Build instructions
- [ ] Contributing guidelines
- [ ] BA2 format reference

**Deliverables**:
- Complete documentation
- Accessible user guides
- Developer onboarding materials

---

### 3.6 Packaging & Distribution

**Priority**: High
**Estimated Effort**: 3-4 days

#### 3.6.1 Build Configuration

- [ ] Release build optimization
- [ ] Strip symbols for smaller binary
- [ ] LTO (Link-Time Optimization)
- [ ] Code size optimization

#### 3.6.2 Portable Build

- [ ] Standalone executable
- [ ] Bundle BSArch.exe with application
- [ ] Bundle resources (icons, translations)
- [ ] Portable config handling (config files in app directory)
- [ ] Include license files (GPL-3.0 for app, MPL-2.0 for BSArch.exe)
- [ ] Create archive structure:
  ```
  unpackrr-rs/
    ├── unpackrr.exe
    ├── BSArch.exe
    ├── LICENSE (GPL-3.0)
    ├── THIRD_PARTY_LICENSES.md (BSArch.exe MPL-2.0)
    ├── resources/
    └── config/ (created on first run)
  ```

#### 3.6.3 Nexus Mods Distribution

- [ ] Create new Nexus Mods page for fork
- [ ] Screenshots and media
- [ ] Installation instructions (simple: extract and run)
- [ ] Changelog
- [ ] Clear attribution to:
  - Original project (KazumaKuun)
  - BSArch.exe (MPL-2.0 licensed tool)
- [ ] Requirements section: none (BSArch.exe bundled)
- [ ] Credits section listing all third-party components

**Deliverables**:
- Portable ZIP archive
- Distribution-ready releases
- Nexus Mods page ready for public testing

**Note**: No installer needed - this is designed to be portable.

---

### Phase 3 Success Criteria

- ✅ Polished UI with smooth animations
- ✅ Comprehensive test suite
- ✅ Complete documentation
- ✅ Portable build ready
- ✅ Ready for public release on Nexus Mods

**Estimated Total Time**: 2-3 weeks

---

## Post-Release Roadmap

### Future Enhancements

#### Features
- [ ] Mod conflict detection
- [ ] Load order optimization suggestions
- [ ] Plugin (ESP/ESM) integration
- [ ] Additional BSArch.exe features (if available)

#### Platform Support
- [ ] Linux support (if BSArch.exe alternatives exist)
- [ ] macOS support (if there's demand and tools available)

#### Automation
- [ ] CLI mode for scripting (still using BSArch.exe)
- [ ] Watch folder mode (auto-extract on new BA2)
- [ ] Integration with mod managers (Vortex, MO2)

**Note**: This is a read-only extractor - no plans for BA2 creation/repacking features.

---

## Development Guidelines

### Daily Workflow

1. **Before Coding**:
   - Review CLAUDE.md guidelines
   - Check current phase tasks
   - Run `cargo test` to ensure baseline

2. **During Development**:
   - Follow Rust 2024 best practices
   - Write tests alongside code
   - Document public APIs
   - Keep commits focused

3. **Before Committing**:
   - Run `cargo fmt`
   - Run `cargo clippy` and fix warnings
   - Run `cargo test`
   - Run `cargo build --release`
   - Update IMPLEMENTATION_PLAN.md progress

### Code Review Checklist

- [ ] Follows CLAUDE.md guidelines
- [ ] Error handling comprehensive
- [ ] No `.unwrap()` in production code
- [ ] Tests written and passing
- [ ] Documentation complete
- [ ] Clippy warnings resolved
- [ ] Memory usage reasonable
- [ ] Windows paths handled correctly

### Testing Strategy

- **Unit tests**: Test individual functions in isolation
- **Integration tests**: Test module interactions
- **Manual testing**: Test with real BA2 files from Nexus Mods
- **Performance testing**: Benchmark critical paths
- **Regression testing**: Ensure fixes don't break existing functionality

---

## Risk Assessment

### High Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| Slint+Tokio integration issues | Could block Phase 1 | Follow async-compat patterns strictly |
| Windows-specific bugs | Could affect UX | Extensive Windows testing; use `dunce` for paths |
| BSArch.exe compatibility issues | Could break extraction | Test with various BA2 types; handle errors gracefully |
| Performance regression vs Python | Could reduce adoption | Benchmark early; optimize hot paths |

### Medium Risk Items

| Risk | Impact | Mitigation |
|------|--------|------------|
| Translation quality | Could affect non-English users | Community contributions; professional review |
| Theme implementation limitations | Could affect visual appeal | Study Slint capabilities early |
| BSArch.exe version compatibility | Different versions may behave differently | Document tested version; include specific version in bundle |

---

## Success Metrics

### Phase 1 (MVP)
- [ ] Application runs without crashes
- [ ] Can extract at least one BA2 successfully
- [ ] Configuration persists across runs

### Phase 2 (Feature Complete)
- [ ] All Python features replicated
- [ ] User testing feedback positive
- [ ] No critical bugs

### Phase 3 (Production Ready)
- [ ] Performance matches or exceeds Python version
- [ ] BSArch.exe integration stable and reliable
- [ ] Test coverage >70%
- [ ] Documentation complete
- [ ] Portable build tested and ready
- [ ] Ready for Nexus Mods release

---

## Timeline Summary

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Phase 1: Foundation | 2-3 weeks | MVP with BSArch.exe integration |
| Phase 2: Features | 3-4 weeks | Full feature parity |
| Phase 3: Polish | 2-3 weeks | Production-ready portable release |
| **Total** | **7-10 weeks** | **Public release on Nexus Mods** |

---

## Next Steps

1. **Review this plan** with stakeholders
2. **Set up development environment** (Phase 1.1)
3. **Create initial project structure** (Phase 1.1)
4. **Begin Phase 1 implementation** starting with configuration management
5. **Establish CI/CD pipeline** for automated testing

---

## Notes

- This plan is a living document; update as implementation progresses
- Estimates are approximate; adjust based on actual velocity
- Prioritize user-facing features over internal optimizations early
- Get user feedback after Phase 1 and Phase 2
- Consider beta releases before final Nexus Mods publication
- Maintain clear attribution to original author (KazumaKuun) throughout project

---

**Last Updated**: 2025-10-21
**Status**: Planning Complete - Ready for Phase 1
**Original Project**: https://github.com/kazum1kun/ba2-batch-unpack-gui
**Current Fork Maintainer**: evildarkarchon
