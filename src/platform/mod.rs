//! Platform-specific functionality
//!
//! This module provides platform-specific implementations for Windows integration
//! and stubs for other platforms.

#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod unix;

// Re-export platform-specific functions
#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
pub use unix::*;
