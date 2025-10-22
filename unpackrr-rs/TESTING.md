# Testing Guide for Unpackrr-rs

This document provides instructions for testing the Rust port in a local development environment.

## Prerequisites

- Rust 2024 edition (install via [rustup](https://rustup.rs/))
- Cargo (comes with Rust)
- BSArch.exe (for integration tests with real BA2 files)

## Running Tests

### Quick Test (All Tests)

```bash
cd unpackrr-rs
cargo test
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Specific Test Modules

```bash
# Config tests only
cargo test config::tests

# BA2 tests only
cargo test ba2::tests

# Error handling tests only
cargo test error::tests
```

### Run Tests with Logging

```bash
RUST_LOG=debug cargo test -- --nocapture
```

## Test Coverage by Phase

### Phase 1.2: Error Handling (3 tests)

```bash
cargo test error::tests
```

Expected tests:
- `test_error_display` - Error display formatting
- `test_ba2_corrupted_check` - BA2 corruption detection
- `test_user_message` - User-friendly error messages

### Phase 1.3: Configuration Management (10 tests)

```bash
cargo test config::tests
```

Expected tests:
- `test_default_config` - Default configuration values
- `test_config_serialization` - JSON serialization round-trip
- `test_postfix_validation` - Invalid postfix rejection
- `test_postfix_validation_success` - Valid config acceptance
- `test_looks_like_regex` - Regex detection heuristic
- `test_should_ignore_file_substring` - Substring matching
- `test_should_ignore_file_regex` - Regex pattern matching
- `test_invalid_regex_validation` - Invalid regex rejection
- `test_log_level_serialization` - LogLevel enum serialization
- (Note: File I/O tests require filesystem access)

### Phase 1.4: BA2 File Format Support (11 tests)

```bash
cargo test ba2::tests
```

Expected tests:

**Header Parsing (7 tests):**
- `test_ba2_magic` - Verify BTDX constant
- `test_header_size` - Verify 24-byte size
- `test_parse_valid_header` - Parse valid header correctly
- `test_parse_invalid_magic` - Reject invalid magic number
- `test_is_general` - Detect GNRL archives
- `test_is_texture` - Detect DX10 archives
- `test_parse_truncated_header` - Handle truncated files

**BSArch Config (4 tests):**
- `test_bsarch_config_default` - Default configuration
- `test_bsarch_config_with_extraction_path` - Custom paths
- `test_bsarch_config_with_temp` - Temp directory mode
- `test_bsarch_config_validation_fails` - Invalid path detection

## Code Quality Checks

### Format Check

```bash
cargo fmt --check
```

### Apply Formatting

```bash
cargo fmt
```

### Clippy (Linting)

```bash
cargo clippy
```

### Clippy with All Warnings

```bash
cargo clippy -- -D warnings
```

## Build Tests

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

### Check Without Building

```bash
cargo check
```

## Integration Testing with Real BA2 Files

### Prerequisites

1. Place `BSArch.exe` in the project root or specify path in config
2. Obtain sample BA2 files (from Fallout 4 or Fallout 76)

### Manual Integration Test

```bash
# Build the project
cargo build --release

# Run with a sample BA2 file (when GUI is implemented)
# ./target/release/unpackrr
```

### Test BA2 Header Parsing

Create a test file `test_ba2_parsing.rs` in `tests/`:

```rust
use unpackrr::ba2::{BA2Header, num_files_in_ba2, is_valid_ba2};
use std::path::PathBuf;

#[test]
fn test_real_ba2_file() {
    let ba2_path = PathBuf::from("path/to/real/file.ba2");

    if !ba2_path.exists() {
        println!("Skipping test - no real BA2 file available");
        return;
    }

    // Test header parsing
    let header = BA2Header::parse(&ba2_path).expect("Failed to parse BA2 header");
    assert_eq!(&header.magic, b"BTDX");
    assert!(header.file_count > 0);

    // Test file count extraction
    let count = num_files_in_ba2(&ba2_path).expect("Failed to get file count");
    assert_eq!(count, header.file_count);

    // Test validation
    assert!(is_valid_ba2(&ba2_path));
}
```

## Expected Test Results (When Dependencies Are Available)

```
running 24 tests
test error::tests::test_error_display ... ok
test error::tests::test_ba2_corrupted_check ... ok
test error::tests::test_user_message ... ok
test config::tests::test_default_config ... ok
test config::tests::test_config_serialization ... ok
test config::tests::test_postfix_validation ... ok
test config::tests::test_postfix_validation_success ... ok
test config::tests::test_looks_like_regex ... ok
test config::tests::test_should_ignore_file_substring ... ok
test config::tests::test_should_ignore_file_regex ... ok
test config::tests::test_invalid_regex_validation ... ok
test config::tests::test_log_level_serialization ... ok
test ba2::tests::test_ba2_magic ... ok
test ba2::tests::test_header_size ... ok
test ba2::tests::test_parse_valid_header ... ok
test ba2::tests::test_parse_invalid_magic ... ok
test ba2::tests::test_is_general ... ok
test ba2::tests::test_is_texture ... ok
test ba2::tests::test_parse_truncated_header ... ok
test ba2::extractor::tests::test_bsarch_config_default ... ok
test ba2::extractor::tests::test_bsarch_config_with_extraction_path ... ok
test ba2::extractor::tests::test_bsarch_config_with_temp ... ok
test ba2::extractor::tests::test_bsarch_config_validation_fails ... ok
test operations::tests::test_parse_size ... ok
test operations::tests::test_parse_size_case_insensitive ... ok
test operations::tests::test_format_size ... ok

test result: ok. 24+ tests passed
```

## Troubleshooting

### Cannot Access crates.io

If you're in a restricted environment:
1. Download the project on a machine with internet access
2. Run `cargo build` to download dependencies
3. Copy the entire project (including `target/` and `Cargo.lock`) to restricted environment
4. Run `cargo test --offline`

### BSArch.exe Not Found

The BSArch.exe integration tests will fail if the executable isn't available. These are expected to fail in CI/CD environments. For local testing:
1. Download BSArch.exe from [official source]
2. Place in project root or configure path in `config.json`

### Slint Compilation Issues

If you encounter Slint compilation errors:
1. Ensure `ui/main.slint` exists
2. Check that `build.rs` is present
3. Verify slint-build dependency in `Cargo.toml`

## Continuous Integration

For CI/CD pipelines, add:

```yaml
name: Rust Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: windows-latest  # or ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Run tests
        run: cargo test --verbose
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt --check
```

## Performance Testing

To benchmark critical paths:

```bash
# Build with optimizations
cargo build --release

# Time header parsing
time ./target/release/unpackrr --parse-only file.ba2
```

## Code Coverage (Optional)

Using `tarpaulin`:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

---

**Note**: This project is under active development. Some tests may require adjustment as features are implemented.
