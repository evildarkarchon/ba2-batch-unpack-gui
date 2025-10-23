//! Path handling utilities
//!
//! This module provides path-related utilities with special handling for Windows:
//! - UNC path support via `dunce` crate
//! - Path canonicalization
//! - Relative/absolute path resolution
//! - Case-insensitive path comparison (Windows)

use crate::error::Result;
use std::path::{Path, PathBuf};

/// Canonicalize a path, handling Windows UNC paths correctly
///
/// This function uses the `dunce` crate to strip the `\\?\` prefix that
/// Windows adds for UNC paths, making paths more user-friendly.
///
/// # Arguments
///
/// * `path` - The path to canonicalize
///
/// # Returns
///
/// A canonicalized `PathBuf` with UNC prefixes stripped on Windows
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use unpackrr::operations::path::canonicalize_path;
///
/// # fn example() -> anyhow::Result<()> {
/// let path = Path::new("C:/Games/../Games/Fallout4");
/// let canonical = canonicalize_path(path)?;
/// assert_eq!(canonical, Path::new("C:/Games/Fallout4"));
/// # Ok(())
/// # }
/// ```
pub fn canonicalize_path(path: &Path) -> Result<PathBuf> {
    let canonical = std::fs::canonicalize(path)?;

    // On Windows, strip the \\?\ prefix using dunce
    #[cfg(windows)]
    {
        Ok(dunce::simplified(&canonical).to_path_buf())
    }

    #[cfg(not(windows))]
    {
        Ok(canonical)
    }
}

/// Resolve a path that may be relative or absolute
///
/// If the path is absolute, returns it as-is (canonicalized).
/// If the path is relative, resolves it relative to the base path.
///
/// # Arguments
///
/// * `path` - The path to resolve
/// * `base` - The base path to resolve relative paths against
///
/// # Returns
///
/// A resolved and canonicalized `PathBuf`
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use unpackrr::operations::path::resolve_path;
///
/// # fn example() -> anyhow::Result<()> {
/// let base = Path::new("C:/Games/Fallout4/Data");
/// let relative = Path::new("../Backups");
/// let resolved = resolve_path(relative, base)?;
/// assert_eq!(resolved, Path::new("C:/Games/Fallout4/Backups"));
/// # Ok(())
/// # }
/// ```
pub fn resolve_path(path: &Path, base: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        canonicalize_path(path)
    } else {
        let full_path = base.join(path);
        canonicalize_path(&full_path)
    }
}

/// Compare two paths for equality, handling case-insensitivity on Windows
///
/// On Windows, paths are compared case-insensitively.
/// On Unix-like systems, paths are compared case-sensitively.
///
/// # Arguments
///
/// * `a` - First path to compare
/// * `b` - Second path to compare
///
/// # Returns
///
/// `true` if the paths are considered equal, `false` otherwise
///
/// # Example
///
/// ```
/// use std::path::Path;
/// use unpackrr::operations::path::paths_equal;
///
/// let path1 = Path::new("C:/Games/Fallout4");
/// let path2 = Path::new("c:/games/fallout4");
///
/// #[cfg(windows)]
/// assert!(paths_equal(path1, path2));
///
/// #[cfg(not(windows))]
/// assert!(!paths_equal(path1, path2));
/// ```
pub fn paths_equal(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        // Windows paths are case-insensitive
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    }

    #[cfg(not(windows))]
    {
        // Unix paths are case-sensitive
        a == b
    }
}

/// Normalize path separators to forward slashes
///
/// Converts Windows backslashes to forward slashes for consistency.
/// Forward slashes work on both Windows and Unix.
///
/// # Arguments
///
/// * `path` - The path to normalize
///
/// # Returns
///
/// A string with normalized path separators
///
/// # Example
///
/// ```
/// use unpackrr::operations::path::normalize_separators;
///
/// let path = "C:\\Games\\Fallout4\\Data";
/// let normalized = normalize_separators(path);
/// assert_eq!(normalized, "C:/Games/Fallout4/Data");
/// ```
pub fn normalize_separators(path: &str) -> String {
    path.replace('\\', "/")
}

/// Check if a path is a valid directory
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// `true` if the path exists and is a directory, `false` otherwise
pub fn is_valid_directory(path: &Path) -> bool {
    path.exists() && path.is_dir()
}

/// Check if a path is a valid file
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// `true` if the path exists and is a file, `false` otherwise
pub fn is_valid_file(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Get the parent directory of a path
///
/// Returns `None` if the path has no parent (e.g., root directory).
///
/// # Arguments
///
/// * `path` - The path to get the parent of
///
/// # Returns
///
/// An `Option<PathBuf>` containing the parent directory, or `None`
pub fn get_parent(path: &Path) -> Option<PathBuf> {
    path.parent().map(|p| p.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_canonicalize_path() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create a subdirectory
        let subdir = temp_path.join("subdir");
        fs::create_dir(&subdir).unwrap();

        // Test canonicalizing the path
        let canonical = canonicalize_path(&subdir).unwrap();
        assert!(canonical.is_absolute());
        assert!(canonical.ends_with("subdir"));
    }

    #[test]
    fn test_resolve_path_absolute() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        let resolved = resolve_path(temp_path, Path::new("/some/base")).unwrap();
        assert!(resolved.is_absolute());
    }

    #[test]
    fn test_resolve_path_relative() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Create subdirectories
        let base = temp_path.join("base");
        let target = temp_path.join("target");
        fs::create_dir(&base).unwrap();
        fs::create_dir(&target).unwrap();

        // Resolve relative path
        let relative = Path::new("../target");
        let resolved = resolve_path(relative, &base).unwrap();
        assert!(resolved.ends_with("target"));
    }

    #[test]
    fn test_paths_equal() {
        let path1 = Path::new("test/path");
        let path2 = Path::new("test/path");
        assert!(paths_equal(path1, path2));

        // Test case sensitivity
        let path3 = Path::new("TEST/PATH");

        #[cfg(windows)]
        assert!(paths_equal(path1, path3));

        #[cfg(not(windows))]
        assert!(!paths_equal(path1, path3));
    }

    #[test]
    fn test_normalize_separators() {
        assert_eq!(
            normalize_separators("C:\\Games\\Fallout4\\Data"),
            "C:/Games/Fallout4/Data"
        );
        assert_eq!(
            normalize_separators("C:/Games/Fallout4/Data"),
            "C:/Games/Fallout4/Data"
        );
    }

    #[test]
    fn test_is_valid_directory() {
        let temp_dir = TempDir::new().unwrap();
        assert!(is_valid_directory(temp_dir.path()));

        let nonexistent = temp_dir.path().join("nonexistent");
        assert!(!is_valid_directory(&nonexistent));
    }

    #[test]
    fn test_is_valid_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        assert!(is_valid_file(&file_path));
        assert!(!is_valid_file(temp_dir.path())); // Directory, not file
    }

    #[test]
    fn test_get_parent() {
        let path = Path::new("/some/path/to/file.txt");
        let parent = get_parent(path).unwrap();
        assert_eq!(parent, Path::new("/some/path/to"));

        // Root has no parent
        #[cfg(unix)]
        {
            let root = Path::new("/");
            assert!(get_parent(root).is_none());
        }
    }
}
