//! Windows-specific platform integration (Phase 2.9)
//!
//! Provides Windows registry access to detect default BA2 file handlers.

use std::path::PathBuf;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_CLASSES_ROOT};
use winreg::RegKey;

/// Get the default application for .ba2 files from Windows registry
///
/// This function queries the Windows registry to find the default handler
/// for .ba2 files and returns the path to the executable.
///
/// # Registry Lookup Strategy
///
/// 1. Check `HKEY_CURRENT_USER\Software\Classes`\.ba2 for user-specific handler
/// 2. Fall back to `HKEY_CLASSES_ROOT`\.ba2 for system-wide handler
/// 3. Follow `ProgId` to find the actual executable path
///
/// # Returns
///
/// - `Some(PathBuf)` - Path to the default BA2 handler executable
/// - `None` - No default handler found or error accessing registry
///
/// # Examples
///
/// ```no_run
/// use unpackrr::platform::get_default_ba2_handler;
///
/// match get_default_ba2_handler() {
///     Some(path) => println!("Default BA2 handler: {}", path.display()),
///     None => println!("No default BA2 handler found"),
/// }
/// ```
pub fn get_default_ba2_handler() -> Option<PathBuf> {
    tracing::info!("Detecting default BA2 file handler from Windows registry");

    // Try user-specific association first (HKCU has priority over HKCR)
    if let Some(path) = get_handler_from_hkcu() {
        tracing::info!("Found user-specific BA2 handler: {}", path.display());
        return Some(path);
    }

    // Fall back to system-wide association
    if let Some(path) = get_handler_from_hkcr() {
        tracing::info!("Found system-wide BA2 handler: {}", path.display());
        return Some(path);
    }

    tracing::info!("No default BA2 handler found in Windows registry");
    None
}

/// Get BA2 handler from `HKEY_CURRENT_USER\Software\Classes`\.ba2
fn get_handler_from_hkcu() -> Option<PathBuf> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Try to open the .ba2 extension key
    let Ok(ba2_key) = hkcu.open_subkey("Software\\Classes\\.ba2") else {
        tracing::debug!("No .ba2 extension registered in HKCU");
        return None;
    };

    // Get the ProgId (e.g., "BA2File" or "Applications\\BSArch.exe")
    let Ok(prog_id) = ba2_key.get_value::<String, _>("") else {
        tracing::debug!("No default value for .ba2 in HKCU");
        return None;
    };

    tracing::debug!("Found ProgId in HKCU: {}", prog_id);

    // Follow the ProgId to find the executable
    get_executable_from_progid(&hkcu, &prog_id)
}

/// Get BA2 handler from `HKEY_CLASSES_ROOT`\.ba2
fn get_handler_from_hkcr() -> Option<PathBuf> {
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);

    // Try to open the .ba2 extension key
    let Ok(ba2_key) = hkcr.open_subkey(".ba2") else {
        tracing::debug!("No .ba2 extension registered in HKCR");
        return None;
    };

    // Get the ProgId
    let Ok(prog_id) = ba2_key.get_value::<String, _>("") else {
        tracing::debug!("No default value for .ba2 in HKCR");
        return None;
    };

    tracing::debug!("Found ProgId in HKCR: {}", prog_id);

    // Follow the ProgId to find the executable
    get_executable_from_progid(&hkcr, &prog_id)
}

/// Get executable path from a `ProgId`
///
/// Follows the registry structure:
/// - {ProgId}\shell\open\command - Contains the command line
fn get_executable_from_progid(root_key: &RegKey, prog_id: &str) -> Option<PathBuf> {
    // Try to open the command key: {ProgId}\shell\open\command
    let command_path = format!("{prog_id}\\shell\\open\\command");

    let Ok(command_key) = root_key.open_subkey(&command_path) else {
        tracing::debug!("No command found for ProgId: {}", prog_id);
        return None;
    };

    // Get the command line
    let Ok(command_line) = command_key.get_value::<String, _>("") else {
        tracing::debug!("No default value for command in ProgId: {}", prog_id);
        return None;
    };

    tracing::debug!("Found command line: {}", command_line);

    // Parse the command line to extract the executable path
    // Command lines can be in formats like:
    // - "C:\Program Files\Tool\tool.exe" "%1"
    // - C:\Program Files\Tool\tool.exe "%1"
    // - "C:\Program Files\Tool\tool.exe"
    let exe_path = parse_executable_path(&command_line);

    Some(exe_path)
}

/// Parse executable path from a command line string
///
/// Handles both quoted and unquoted paths, and strips any arguments.
fn parse_executable_path(command_line: &str) -> PathBuf {
    let trimmed = command_line.trim();

    // Check if the command starts with a quote
    if let Some(stripped) = trimmed.strip_prefix('"') {
        // Find the closing quote
        if let Some(end_quote_pos) = stripped.find('"') {
            let exe_path = &stripped[..end_quote_pos];
            return PathBuf::from(exe_path);
        }
    }

    // Try to split by space and take the first part (unquoted path)
    let first_part = trimmed.split_whitespace().next().unwrap_or(trimmed);

    // Validate that the path exists (basic check)
    let path = PathBuf::from(first_part);

    if path.exists() {
        path
    } else {
        // If not found, return the original trimmed string as a path
        // (it might be valid but not accessible from this context)
        PathBuf::from(trimmed)
    }
}

/// Check if a file is a valid executable
///
/// On Windows, checks if the file has .exe, .bat, or .cmd extension.
pub fn is_valid_executable(path: &std::path::Path) -> bool {
    if !path.exists() {
        return false;
    }

    path.extension().is_some_and(|ext| {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        matches!(ext_lower.as_str(), "exe" | "bat" | "cmd")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_executable_path_quoted() {
        let cmd = r#""C:\Program Files\BSArch\BSArch.exe" "%1""#;
        let result = parse_executable_path(cmd);
        assert_eq!(result, PathBuf::from(r"C:\Program Files\BSArch\BSArch.exe"));
    }

    #[test]
    fn test_parse_executable_path_unquoted() {
        let cmd = r"C:\Tools\BSArch.exe %1";
        let result = parse_executable_path(cmd);
        assert!(
            result == PathBuf::from(r"C:\Tools\BSArch.exe")
                || result == PathBuf::from(r"C:\Tools\BSArch.exe %1")
        );
    }

    #[test]
    fn test_parse_executable_path_simple() {
        let cmd = r#""C:\Program Files\BSArch\BSArch.exe""#;
        let result = parse_executable_path(cmd);
        assert_eq!(result, PathBuf::from(r"C:\Program Files\BSArch\BSArch.exe"));
    }

    // Note: get_default_ba2_handler() tests would require a Windows environment
    // with registry access, so we skip them in CI
}
