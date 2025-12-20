//! Unix/Linux/macOS platform stubs (Phase 2.9)
//!
//! Provides stub implementations for non-Windows platforms.
//! These functions return None or appropriate defaults since BA2 files
//! are primarily a Windows gaming format.

use anyhow::Result;
use std::path::PathBuf;

/// Get the default application for .ba2 files (stub for non-Windows platforms)
///
/// On Unix-like systems, BA2 files don't have system-wide file associations
/// in the same way as Windows. This function always returns `None`.
///
/// # Returns
///
/// Always returns `Ok(None)` on non-Windows platforms.
///
/// # Examples
///
/// ```
/// use unpackrr::platform::get_default_ba2_handler;
///
/// let handler = get_default_ba2_handler().unwrap();
/// assert_eq!(handler, None); // Always None on Unix
/// ```
pub fn get_default_ba2_handler() -> Result<Option<PathBuf>> {
    tracing::debug!("get_default_ba2_handler() called on non-Windows platform - returning None");
    Ok(None)
}

/// Check if a file is a valid executable (Unix implementation)
///
/// On Unix-like systems, checks if the file exists and has execute permissions.
pub fn is_valid_executable(path: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if !path.exists() {
            return false;
        }

        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            // Check if the file has execute permission for owner, group, or others
            permissions.mode() & 0o111 != 0
        } else {
            false
        }
    }

    #[cfg(not(unix))]
    {
        // Fallback for other non-Windows, non-Unix platforms (unlikely)
        path.exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_ba2_handler_returns_none() {
        let result = get_default_ba2_handler().unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_is_valid_executable_nonexistent() {
        let path = PathBuf::from("/nonexistent/file");
        assert!(!is_valid_executable(&path));
    }
}
