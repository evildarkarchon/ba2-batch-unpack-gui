# Unpackrr - BA2 Batch Unpacker (Rust Edition)

**High-performance BA2 archive manager with Fluent Design UI**

<div align="center">

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Slint](https://img.shields.io/badge/GUI-Slint-blueviolet.svg)](https://slint.dev/)

</div>

---

## About

Unpackrr is a powerful graphical application for managing Bethesda Archive 2 (BA2) files used in Fallout 4 and Fallout 76. It automatically extracts small BA2 archives to help you stay under the game's BA2 file limit (typically 255 files), preventing crashes and improving mod compatibility.

**This is a Rust port** of the [original Python-based Unpackrr](https://github.com/kazum1kun/ba2-batch-unpack-gui) by [KazumaKuun](https://github.com/kazum1kun), rewritten for:
- **Better Performance**: Native compilation and async I/O for faster processing
- **Enhanced Reliability**: Rust's memory safety prevents crashes
- **Modern UI**: Fluent Design interface with light/dark themes
- **Cross-Platform Readiness**: Built with portability in mind

**Original Author**: KazumaKuun / Southwest Codeworks
**Current Maintainer**: [evildarkarchon](https://github.com/evildarkarchon)
**License**: GPL-3.0 (application), MPL-2.0 (bundled BSArch.exe)

---

## Features

### Extraction Management
- ✅ **Automatic BA2 Extraction** - Keep your load order under the BA2 limit
- ✅ **Smart Threshold Calculation** - Auto-calculate size thresholds or set manually
- ✅ **Flexible Filtering** - Postfix-based selection with regex support for ignoring files
- ✅ **Real-Time Progress** - Track extraction progress, speed, and ETA
- ✅ **Pause/Resume/Cancel** - Full control over extraction operations
- ✅ **Automatic Backup** - Save original BA2s before extraction (configurable)

### File Validation
- ✅ **Quick Scan** - List BA2 contents to detect corruption
- ✅ **Deep Scan** - Extract to temp directory for thorough validation
- ✅ **Batch Checking** - Scan entire mod folders at once
- ✅ **Detailed Reports** - Identify corrupted archives before they cause issues

### User Interface
- ✅ **Fluent Design** - Modern, polished interface following Microsoft Fluent principles
- ✅ **Theme Support** - Light, dark, and system-based themes
- ✅ **Custom Accent Colors** - Personalize your experience
- ✅ **Sortable Tables** - Organize BA2 files by name, size, file count, or mod
- ✅ **Context Menus** - Right-click for quick actions
- ✅ **Native File Dialogs** - System-native folder selection

### Advanced Features
- ✅ **External Tool Integration** - Open BA2s in your preferred BA2 viewer
- ✅ **Windows Registry Detection** - Auto-detect default BA2 handler
- ✅ **Update Checking** - Stay informed about new releases
- ✅ **Comprehensive Error Handling** - User-friendly messages with recovery suggestions
- ✅ **Retry Logic** - Automatic retry with exponential backoff for transient failures
- ✅ **Detailed Logging** - Daily rotating logs for troubleshooting

---

## Installation

### Download

**Portable Release** (Recommended):
1. Download the latest release from the [Releases page](https://github.com/evildarkarchon/ba2-batch-unpack-gui/releases)
2. Extract the ZIP archive to your desired location
3. Run `unpackrr.exe` - no installation required!

**What's Included**:
```
unpackrr-rs/
  ├── unpackrr.exe          # Main application
  ├── BSArch.exe            # BA2 extraction tool (MPL-2.0 licensed)
  ├── LICENSE               # GPL-3.0 license
  ├── THIRD_PARTY_LICENSES.md
  └── README.md             # This file
```

### Building from Source

**Requirements**:
- Rust 1.85+ (2024 edition)
- Cargo (included with Rust)
- Git

**Steps**:
```bash
# Clone the repository
git clone https://github.com/evildarkarchon/ba2-batch-unpack-gui.git
cd ba2-batch-unpack-gui/unpackrr-rs

# Build release binary
cargo build --release

# Binary will be at: target/release/unpackrr.exe (Windows) or target/release/unpackrr (Linux/macOS)
```

**Note**: You'll need to obtain `BSArch.exe` separately from the [TES5Edit project](https://github.com/TES5Edit/TES5Edit) and place it in the same directory as the executable.

---

## Usage

### Quick Start

**Extract BA2 Files**:
1. Launch Unpackrr
2. Click **Browse** or drag-and-drop your Fallout 4 mod folder
   - **Mod Organizer 2**: Open → Open Mods folder
   - **Vortex**: Open → Open Mod Staging Folder
3. Preview the BA2 files that will be extracted
4. (Optional) Click **Auto** to calculate optimal size threshold, or enter manually
5. Click **Start Extraction** and wait for completion

**Check for Corrupted Files**:
1. Click the **Check Files** tab
2. Select your mod folder
3. Click **Start**
4. (Optional) Enable **Deep Scan** for thorough checking

### Settings

Access settings via the gear icon in the sidebar.

**Extraction Settings**:
- **Postfixes**: File endings to include (e.g., `- Main.ba2`)
- **Ignored Files**: Patterns to exclude (supports regex in `{pattern}` format)
- **Ignore Bad Files**: Skip corrupted archives during extraction
- **Automatic Backup**: Save original BA2s to backup folder

**Personalization**:
- **Theme**: Light, Dark, or System
- **Accent Color**: Customize highlight colors
- **Language**: English (additional languages may be added later)

**Advanced Settings**:
- **Show Debug Log**: Enable detailed logging output
- **Extraction Path**: Where to extract files (default: in-place)
- **Backup Path**: Where to save backups (default: `backup/` in mod folder)
- **External BA2 Tool**: Path to your preferred BA2 viewer

---

## Technical Details

### Architecture

Unpackrr is built on:
- **Rust 2024 Edition** - Modern, safe systems programming
- **Slint UI Framework** - Declarative, hardware-accelerated GUI
- **Tokio Async Runtime** - Concurrent I/O operations
- **BSArch.exe** - Battle-tested BA2 extraction (MPL-2.0 licensed)

**Key Technologies**:
- `async-compat` - Slint + Tokio integration
- `reqwest` - Update checking via GitHub API
- `tracing` - Comprehensive logging
- `serde` - Configuration serialization
- `regex` - Advanced file filtering
- `rayon` - Parallel BA2 scanning

### BSArch.exe

This application uses [BSArch.exe](https://github.com/TES5Edit/TES5Edit) for BA2 extraction. BSArch is a mature, reliable tool developed by the TES5Edit team, licensed under the Mozilla Public License 2.0 (MPL-2.0).

**Why not reimplement extraction in Rust?**
- BSArch.exe is years ahead in terms of BA2 format support and edge case handling
- It's actively maintained and updated for new Bethesda game releases
- Focus our efforts on user experience rather than reinventing the wheel
- MPL-2.0 license permits redistribution with proper attribution

### File Selection Logic

A BA2 file is extracted if it meets **all** of these criteria:
1. Contains at least one postfix from the "Postfixes" list
2. Does **not** contain any pattern from the "Ignored Files" list
3. Is smaller than the specified threshold (if set)

**Regex Support**:
Wrap patterns in curly braces for regex matching:
- `{.*[dD]iamond.*}` - Matches files containing "diamond" or "Diamond"
- `{^Mod.*Main\.ba2$}` - Matches files starting with "Mod" and ending with "Main.ba2"

**Note**: Regex patterns use `fullmatch()`, meaning they're anchored at start and end (`^pattern$`).

---

## Configuration

Settings are stored in JSON format at:
- **Windows**: `%APPDATA%\Unpackrr\config\config.json`
- **Linux**: `~/.config/unpackrr/config/config.json`
- **macOS**: `~/Library/Application Support/com.unpackrr.app/config/config.json`

Logs are stored at:
- **Windows**: `%APPDATA%\Unpackrr\logs\`
- **Linux**: `~/.local/share/unpackrr/logs/`
- **macOS**: `~/Library/Application Support/com.unpackrr.app/logs/`

Log files rotate daily: `unpackrr-YYYY-MM-DD.log`

---

## Troubleshooting

### Common Issues

**Application won't start**:
- Ensure `BSArch.exe` is in the same directory as `unpackrr.exe`
- Check logs in the data directory for detailed error messages
- Try running from command line to see startup errors

**Extraction fails**:
- Verify BA2 files aren't corrupted using the Check Files screen
- Ensure you have write permissions to the mod folder
- Check that BSArch.exe is not blocked by antivirus software

**Performance issues**:
- Large mod collections (1000+ BA2s) may take time to scan
- Enable logging to identify bottlenecks
- Consider using manual threshold instead of Auto for huge collections

### Reporting Bugs

Please report issues on [GitHub Issues](https://github.com/evildarkarchon/ba2-batch-unpack-gui/issues) with:
1. Unpackrr version (shown in Settings → About)
2. Operating system and version
3. Steps to reproduce the issue
4. Relevant log entries from the log file
5. Screenshots (if UI-related)

---

## Development

### Project Structure

```
unpackrr-rs/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── error.rs             # Error types and handling
│   ├── config/              # Configuration management
│   ├── ba2/                 # BA2 format support
│   ├── operations/          # File operations (scan, extract, validate)
│   ├── models/              # Data models
│   ├── platform/            # Platform-specific code (Windows/Unix)
│   ├── logging/             # Logging infrastructure
│   └── ui/                  # UI integration and callbacks
├── ui/
│   └── main.slint          # Slint UI definition
├── tests/                   # Integration tests
├── Cargo.toml              # Dependencies and build configuration
└── build.rs                # Build script (Slint compilation)
```

### Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Follow Rust 2024 best practices (see `CLAUDE.md` for guidelines)
4. Run `cargo fmt` and `cargo clippy` before committing
5. Ensure all tests pass (`cargo test`)
6. Submit a pull request

**Development Guidelines**:
- Follow the patterns in `CLAUDE.md` and `IMPLEMENTATION_PLAN.md`
- Add tests for new functionality
- Document public APIs with `///` doc comments
- Keep commits focused and well-described

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Build and run
cargo run

# Release build
cargo build --release
```

---

## Credits

### Original Project
**Unpackrr** was originally created by **KazumaKuun** (Southwest Codeworks):
- GitHub: [kazum1kun](https://github.com/kazum1kun)
- Nexus Mods: [Original Unpackrr](https://www.nexusmods.com/fallout4/mods/79593)
- Ko-fi: [Support the original developer](https://ko-fi.com/kazumakuun)

### Current Rust Port
**Maintained by**: [evildarkarchon](https://github.com/evildarkarchon)
**Repository**: [ba2-batch-unpack-gui](https://github.com/evildarkarchon/ba2-batch-unpack-gui)

### Third-Party Components
- **BSArch.exe**: [TES5Edit Team](https://github.com/TES5Edit/TES5Edit) - MPL-2.0 License
- **Slint**: [Slint](https://slint.dev/) - GPL-3.0 Compatible
- **Rust Crates**: See `Cargo.toml` for full dependency list

---

## License

This project is licensed under the **GNU General Public License v3.0** (GPL-3.0).
See [LICENSE](LICENSE) for full terms.

**Bundled Third-Party Software**:
- **BSArch.exe**: Licensed under Mozilla Public License 2.0 (MPL-2.0)
- See [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) for details

---

## Changelog

### Version 0.1.0 (In Development)
- Complete Rust port of original Python application
- Fluent Design UI with Slint framework
- Enhanced performance with async I/O
- Real-time progress tracking with speed and ETA
- Pause/Resume/Cancel extraction
- Advanced error handling with retry logic
- Theme support (light/dark/system)
- Windows registry integration for default BA2 handler
- Comprehensive logging with daily rotation
- Update checking via GitHub API

---

## Roadmap

### Planned Features
- UI animations and polish

### Not Planned
- BA2 creation/packing features (use BSArch.exe or Creation Kit directly)
- Mod conflict detection (scope creep; better suited for mod managers)
- ESM/ESP plugin management (out of scope)

---

## Acknowledgments

Special thanks to:
- **KazumaKuun** for creating the original Unpackrr and inspiring this port
- **TES5Edit Team** for BSArch.exe, the backbone of BA2 extraction
- **Slint Team** for the excellent GUI framework
- **Rust Community** for amazing tools and libraries
- **Fallout 4 Modding Community** for feedback and support

---

<div align="center">

**Made with ❤️ and Rust**

[Report Issue](https://github.com/evildarkarchon/ba2-batch-unpack-gui/issues) · [Request Feature](https://github.com/evildarkarchon/ba2-batch-unpack-gui/issues) · [Nexus Mods](https://www.nexusmods.com/fallout4/mods/79593)

</div>
