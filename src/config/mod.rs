//! Configuration management for Unpackrr-rs
//!
//! This module handles loading, saving, and validating application configuration.
//! Configuration is stored in JSON format and includes settings for:
//! - Extraction behavior (postfixes, ignored files, auto backup)
//! - Appearance (theme, language, accent color)
//! - Advanced settings (debug mode, paths, external tools)
//! - Update checking preferences

use crate::error::{ConfigError, Result};
use directories::ProjectDirs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Extraction-related settings
    pub extraction: ExtractionConfig,

    /// Saved user settings
    pub saved: SavedConfig,

    /// Appearance and personalization settings
    pub appearance: AppearanceConfig,

    /// Advanced settings
    pub advanced: AdvancedConfig,

    /// Update checking settings
    pub update: UpdateConfig,
}

/// Extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// BA2 file postfixes to process (e.g., "main.ba2", "textures.ba2")
    /// Files must end with .ba2
    #[serde(default = "default_postfixes")]
    pub postfixes: Vec<String>,

    /// Files to ignore (exact match, substring, or regex)
    #[serde(default)]
    pub ignored_files: Vec<String>,

    /// Ignore corrupted BA2 files
    #[serde(default = "default_true")]
    pub ignore_bad_files: bool,

    /// Automatically backup BA2 files before extraction
    #[serde(default = "default_true")]
    pub auto_backup: bool,
}

/// Saved user settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedConfig {
    /// Last used directory
    #[serde(default)]
    pub directory: String,

    /// Last used size threshold (in bytes)
    #[serde(default)]
    pub threshold: u64,
}

/// Appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    /// Theme mode: "light", "dark", or "system"
    pub theme_mode: String,

    /// Accent color (hex format)
    pub accent_color: String,

    /// Language: "auto", "en", "zh-CN", "zh-TW"
    pub language: String,
}

/// Advanced configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    /// Show debug information in UI
    #[serde(default)]
    pub show_debug: bool,

    /// Log level for debugging
    #[serde(default)]
    pub log_level: LogLevel,

    /// First launch flag
    #[serde(default = "default_true")]
    pub first_launch: bool,

    /// Custom extraction path (empty = use default)
    #[serde(default)]
    pub extraction_path: String,

    /// Custom backup path (empty = use default)
    #[serde(default)]
    pub backup_path: String,

    /// External BA2 tool path (empty = use bundled BSArch.exe)
    #[serde(default)]
    pub ext_ba2_exe: String,
}

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[derive(Default)]
pub enum LogLevel {
    /// Fatal errors (most critical)
    Fatal = 0,
    /// Error messages
    Error = 1,
    /// Warning messages
    #[default]
    Warning = 2,
    /// Informational messages
    Info = 3,
    /// Debug messages
    Debug = 4,
    /// Trace messages (most verbose)
    Trace = 5,
}

/// Update checking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Check for updates at startup
    pub check_at_startup: bool,
}

// Default value helpers for serde
fn default_postfixes() -> Vec<String> {
    vec![
        "main.ba2".to_string(),
        "materials.ba2".to_string(),
        "misc.ba2".to_string(),
        "scripts.ba2".to_string(),
    ]
}

const fn default_true() -> bool {
    true
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            postfixes: default_postfixes(),
            ignored_files: Vec::new(),
            ignore_bad_files: true,
            auto_backup: true,
        }
    }
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme_mode: "dark".to_string(),
            accent_color: "#0078D4".to_string(), // Fluent Design default blue
            language: "auto".to_string(),
        }
    }
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            show_debug: false,
            log_level: LogLevel::Warning,
            first_launch: true,
            extraction_path: String::new(),
            backup_path: String::new(),
            ext_ba2_exe: String::new(),
        }
    }
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            check_at_startup: true,
        }
    }
}

impl AppConfig {
    /// Get the default configuration directory path
    pub fn config_dir() -> Result<PathBuf> {
        ProjectDirs::from("com", "evildarkarchon", "unpackrr")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .ok_or_else(|| {
                ConfigError::ValidationFailed("Could not determine config directory".to_string())
                    .into()
            })
    }

    /// Get the configuration file path
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.json"))
    }

    /// Load configuration from file, or create default if not exists
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if !config_path.exists() {
            tracing::info!(
                "Configuration file not found, creating default at: {}",
                config_path.display()
            );
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path).map_err(|e| ConfigError::LoadFailed {
            path: config_path.clone(),
            source: e,
        })?;

        let config: Self = serde_json::from_str(&content)
            .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;

        config.validate()?;

        tracing::info!(
            "Configuration loaded successfully from: {}",
            config_path.display()
        );
        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;

        // Validate before saving
        self.validate()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ConfigError::SaveFailed {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        // Serialize with pretty formatting
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;

        fs::write(&config_path, content).map_err(|e| ConfigError::SaveFailed {
            path: config_path.clone(),
            source: e,
        })?;

        tracing::info!(
            "Configuration saved successfully to: {}",
            config_path.display()
        );
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate postfixes - all must end with .ba2
        for postfix in &self.extraction.postfixes {
            if !Path::new(postfix)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("ba2"))
            {
                return Err(ConfigError::ValidationFailed(format!(
                    "Postfix '{postfix}' must end with .ba2"
                ))
                .into());
            }
        }

        // Validate threshold is non-negative (u64 is always non-negative, but check for clarity)
        // This is mainly for documentation purposes

        // Validate paths if specified
        if !self.advanced.extraction_path.is_empty() {
            let path = resolve_path(&self.advanced.extraction_path)?;
            if !path.exists() {
                tracing::warn!("Custom extraction path does not exist: {}", path.display());
            }
        }

        if !self.advanced.backup_path.is_empty() {
            let path = resolve_path(&self.advanced.backup_path)?;
            if !path.exists() {
                tracing::warn!("Custom backup path does not exist: {}", path.display());
            }
        }

        if !self.advanced.ext_ba2_exe.is_empty() {
            let path = resolve_path(&self.advanced.ext_ba2_exe)?;
            if !path.exists() {
                return Err(ConfigError::InvalidPath(path).into());
            }
        }

        // Validate ignored files regex patterns if they look like regex
        for pattern in &self.extraction.ignored_files {
            if looks_like_regex(pattern)
                && let Err(e) = Regex::new(pattern)
            {
                return Err(ConfigError::InvalidRegex {
                    pattern: pattern.clone(),
                    source: e,
                }
                .into());
            }
        }

        Ok(())
    }

    /// Get compiled regex patterns for ignored files
    /// Results are cached globally
    pub fn get_ignored_patterns(&self) -> Result<Vec<Regex>> {
        let mut patterns = Vec::new();
        for pattern in &self.extraction.ignored_files {
            if looks_like_regex(pattern) {
                let regex = Regex::new(pattern).map_err(|e| ConfigError::InvalidRegex {
                    pattern: pattern.clone(),
                    source: e,
                })?;
                patterns.push(regex);
            }
        }
        Ok(patterns)
    }

    /// Check if a file should be ignored based on configured patterns
    ///
    /// This method checks both the file name and full path against:
    /// - Exact path matches
    /// - Substring matches
    /// - Regex patterns
    ///
    /// # Arguments
    ///
    /// * `path` - The full path to the file to check
    ///
    /// # Returns
    ///
    /// `true` if the file should be ignored, `false` otherwise
    pub fn should_ignore_file(&self, path: &Path) -> bool {
        // Get file name for checking
        let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
            return false;
        };

        // Check exact path match
        if self
            .extraction
            .ignored_files
            .contains(&path.to_string_lossy().to_string())
        {
            return true;
        }

        // Get regex patterns (ignore errors, just skip regex matching if it fails)
        let regex_patterns = self.get_ignored_patterns().unwrap_or_default();

        // Use the standalone function for the actual checking logic
        should_ignore_file(file_name, &self.extraction.ignored_files, &regex_patterns)
    }
}

/// Resolve a path to an absolute path, handling Windows UNC paths correctly
pub fn resolve_path(path: &str) -> Result<PathBuf> {
    if path.is_empty() {
        return Err(ConfigError::InvalidPath(PathBuf::new()).into());
    }

    let path_buf = PathBuf::from(path);

    // Use dunce to handle Windows UNC paths correctly
    let resolved = if path_buf.is_absolute() {
        dunce::canonicalize(&path_buf).unwrap_or(path_buf)
    } else {
        // Resolve relative to current directory
        let current_dir = std::env::current_dir().map_err(|e| {
            ConfigError::ValidationFailed(format!("Cannot get current directory: {e}"))
        })?;
        let full_path = current_dir.join(&path_buf);
        dunce::canonicalize(&full_path).unwrap_or(full_path)
    };

    Ok(resolved)
}

/// Check if a string looks like a regex pattern
///
/// This is a simple heuristic to avoid compiling plain strings as regex.
/// Patterns containing regex metacharacters are likely regex patterns.
fn looks_like_regex(pattern: &str) -> bool {
    pattern.contains('[')
        || pattern.contains(']')
        || pattern.contains('(')
        || pattern.contains(')')
        || pattern.contains('*')
        || pattern.contains('+')
        || pattern.contains('?')
        || pattern.contains('|')
        || pattern.contains('^')
        || pattern.contains('$')
        || pattern.contains('\\')
        || pattern.contains('.')
}

/// Check if a file should be ignored based on the configured patterns
pub fn should_ignore_file(
    file_name: &str,
    ignored_files: &[String],
    regex_patterns: &[Regex],
) -> bool {
    // First check exact matches and substrings
    for pattern in ignored_files {
        if !looks_like_regex(pattern) {
            // Simple substring match
            if file_name.contains(pattern) {
                return true;
            }
        }
    }

    // Then check regex patterns
    for regex in regex_patterns {
        if regex.is_match(file_name) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.appearance.theme_mode, "dark");
        assert!(config.update.check_at_startup);
        assert_eq!(config.advanced.log_level, LogLevel::Warning);
        assert!(config.advanced.first_launch);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: AppConfig = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized.appearance.language, config.appearance.language);
        assert_eq!(deserialized.advanced.log_level, LogLevel::Warning);
    }

    #[test]
    fn test_postfix_validation() {
        let mut config = AppConfig::default();
        config.extraction.postfixes.push("invalid.txt".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_postfix_validation_success() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_looks_like_regex() {
        assert!(looks_like_regex(".*\\.ba2$"));
        assert!(looks_like_regex("[0-9]+"));
        assert!(looks_like_regex("foo|bar"));
        assert!(!looks_like_regex("simple_string"));
        assert!(!looks_like_regex("file_name"));
    }

    #[test]
    fn test_should_ignore_file_substring() {
        let ignored = vec!["test".to_string(), "debug".to_string()];
        let patterns = vec![];

        assert!(should_ignore_file("test_file.ba2", &ignored, &patterns));
        assert!(should_ignore_file(
            "debug_textures.ba2",
            &ignored,
            &patterns
        ));
        assert!(!should_ignore_file("main.ba2", &ignored, &patterns));
    }

    #[test]
    fn test_should_ignore_file_regex() {
        let ignored = vec![".*test.*".to_string()];
        let patterns = vec![Regex::new(".*test.*").unwrap()];

        assert!(should_ignore_file("test_file.ba2", &ignored, &patterns));
        assert!(should_ignore_file("my_test_mod.ba2", &ignored, &patterns));
        assert!(!should_ignore_file("main.ba2", &ignored, &patterns));
    }

    #[test]
    fn test_invalid_regex_validation() {
        let mut config = AppConfig::default();
        config.extraction.ignored_files.push("[invalid".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_log_level_serialization() {
        let level = LogLevel::Debug;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"DEBUG\"");

        let deserialized: LogLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, LogLevel::Debug);
    }
}
