//! Log viewer functionality for displaying and filtering application logs
//!
//! This module provides functionality to read, parse, and filter log files
//! for display in the UI. It supports:
//! - Reading from daily rotating log files
//! - Parsing structured log entries
//! - Filtering by log level
//! - Real-time log updates via file watching

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tracing::Level;

/// Represents a single log entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// The full raw log line
    pub raw_line: String,
    /// Parsed log level (if available)
    pub level: Option<LogLevel>,
    /// Timestamp (if available)
    pub timestamp: Option<String>,
    /// Log target/module (if available)
    pub target: Option<String>,
    /// The actual message
    pub message: String,
}

/// Log levels for filtering and display
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Trace-level logging (most verbose)
    Trace,
    /// Debug-level logging
    Debug,
    /// Info-level logging
    Info,
    /// Warning-level logging
    Warn,
    /// Error-level logging (least verbose)
    Error,
}

impl LogLevel {
    /// Convert from tracing::Level
    pub fn from_tracing_level(level: &Level) -> Self {
        match *level {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        }
    }

    /// Convert to display string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }

    /// Get color code for this level (as RGB hex string for Slint)
    pub fn color(&self) -> &'static str {
        match self {
            LogLevel::Error => "#FF0000",      // Red
            LogLevel::Warn => "#FF6A5B",       // Light red/orange
            LogLevel::Info => "#FFFFFF",       // White
            LogLevel::Debug => "#A0A0A0",      // Light gray
            LogLevel::Trace => "#808080",      // Gray
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl LogEntry {
    /// Parse a log line into a structured entry
    ///
    /// Expected format from tracing_subscriber:
    /// `2025-01-22T10:30:45.123456Z  INFO unpackrr::operations::scan: Starting BA2 scan in: /path/to/folder`
    pub fn parse(line: String) -> Self {
        let mut level = None;
        let mut timestamp = None;
        let mut target = None;
        let mut message = line.clone();

        // Try to parse structured log format
        // Format: TIMESTAMP LEVEL TARGET: MESSAGE
        // Note: there may be multiple spaces between timestamp and level
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 3 {
            // Try to parse timestamp (ISO 8601 format)
            if parts[0].contains('T') && (parts[0].contains('Z') || parts[0].contains('+')) {
                timestamp = Some(parts[0].to_string());

                // Parse log level (parts[1] after split_whitespace())
                level = match parts[1] {
                    "TRACE" => Some(LogLevel::Trace),
                    "DEBUG" => Some(LogLevel::Debug),
                    "INFO" => Some(LogLevel::Info),
                    "WARN" => Some(LogLevel::Warn),
                    "ERROR" => Some(LogLevel::Error),
                    _ => None,
                };

                if level.is_some() && parts.len() >= 3 {
                    // Parse target and message
                    // Find the position after timestamp and level in the original string
                    let after_level = line.find(parts[1])
                        .and_then(|pos| Some(pos + parts[1].len()))
                        .unwrap_or(0);

                    let rest = line[after_level..].trim_start();

                    // Check if there's a colon-space indicating target: message format
                    // We look for ": " to avoid matching "::" in module paths
                    if let Some(colon_pos) = rest.find(": ") {
                        target = Some(rest[..colon_pos].to_string());
                        message = rest[colon_pos + 2..].to_string(); // Skip ": "
                    } else {
                        message = rest.to_string();
                    }
                }
            }
        }

        Self {
            raw_line: line,
            level,
            timestamp,
            target,
            message: message.trim().to_string(),
        }
    }

    /// Check if this entry matches the given filter level
    /// Returns true if the entry's level is >= the filter level
    pub fn matches_filter(&self, filter: Option<LogLevel>) -> bool {
        match (self.level, filter) {
            (Some(entry_level), Some(filter_level)) => entry_level >= filter_level,
            (None, _) => true,  // Always show unparseable lines
            (_, None) => true,  // No filter, show all
        }
    }
}

/// Log viewer that reads and manages log entries
pub struct LogViewer {
    /// All loaded log entries
    entries: Vec<LogEntry>,
    /// Current filter level (None = show all)
    filter_level: Option<LogLevel>,
}

impl LogViewer {
    /// Create a new log viewer
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            filter_level: None,
        }
    }

    /// Load logs from the current log file
    pub fn load_logs(&mut self) -> Result<()> {
        let log_dir = crate::logging::get_log_dir()?;

        // Get today's log file
        let log_file = log_dir.join("unpackrr.log");

        if !log_file.exists() {
            // No logs yet, that's okay
            return Ok(());
        }

        self.load_from_file(&log_file)?;
        Ok(())
    }

    /// Load logs from a specific file
    fn load_from_file(&mut self, path: &PathBuf) -> Result<()> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open log file: {}", path.display()))?;

        let reader = BufReader::new(file);

        self.entries.clear();

        for line in reader.lines() {
            let line = line.with_context(|| "Failed to read log line")?;
            self.entries.push(LogEntry::parse(line));
        }

        Ok(())
    }

    /// Get filtered entries based on current filter level
    pub fn get_filtered_entries(&self) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.matches_filter(self.filter_level))
            .cloned()
            .collect()
    }

    /// Get all entries (unfiltered)
    pub fn get_all_entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Set the filter level
    pub fn set_filter(&mut self, level: Option<LogLevel>) {
        self.filter_level = level;
    }

    /// Get current filter level
    pub fn get_filter(&self) -> Option<LogLevel> {
        self.filter_level
    }

    /// Clear all loaded entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get the number of entries at each log level
    pub fn get_level_counts(&self) -> (usize, usize, usize, usize, usize) {
        let mut trace = 0;
        let mut debug = 0;
        let mut info = 0;
        let mut warn = 0;
        let mut error = 0;

        for entry in &self.entries {
            match entry.level {
                Some(LogLevel::Trace) => trace += 1,
                Some(LogLevel::Debug) => debug += 1,
                Some(LogLevel::Info) => info += 1,
                Some(LogLevel::Warn) => warn += 1,
                Some(LogLevel::Error) => error += 1,
                None => {} // Unparseable lines don't count
            }
        }

        (trace, debug, info, warn, error)
    }

    /// Refresh logs from disk
    pub fn refresh(&mut self) -> Result<()> {
        self.load_logs()
    }
}

impl Default for LogViewer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_structured_log() {
        let line = "2025-01-22T10:30:45.123456Z  INFO unpackrr::operations::scan: Starting BA2 scan".to_string();
        let entry = LogEntry::parse(line);

        assert_eq!(entry.level, Some(LogLevel::Info));
        assert_eq!(entry.timestamp, Some("2025-01-22T10:30:45.123456Z".to_string()));
        assert_eq!(entry.target, Some("unpackrr::operations::scan".to_string()));
        assert_eq!(entry.message, "Starting BA2 scan");
    }

    #[test]
    fn test_parse_error_log() {
        let line = "2025-01-22T10:30:45.123456Z ERROR unpackrr::error: File not found".to_string();
        let entry = LogEntry::parse(line);

        assert_eq!(entry.level, Some(LogLevel::Error));
        assert_eq!(entry.target, Some("unpackrr::error".to_string()));
    }

    #[test]
    fn test_parse_unstructured_log() {
        let line = "Some random log message".to_string();
        let entry = LogEntry::parse(line.clone());

        assert_eq!(entry.level, None);
        assert_eq!(entry.timestamp, None);
        assert_eq!(entry.raw_line, line);
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn test_matches_filter() {
        let info_entry = LogEntry::parse("2025-01-22T10:30:45.123456Z  INFO test: message".to_string());
        let debug_entry = LogEntry::parse("2025-01-22T10:30:45.123456Z DEBUG test: message".to_string());

        // Info filter should show info and above
        assert!(info_entry.matches_filter(Some(LogLevel::Info)));
        assert!(!debug_entry.matches_filter(Some(LogLevel::Info)));

        // Debug filter should show debug and above
        assert!(info_entry.matches_filter(Some(LogLevel::Debug)));
        assert!(debug_entry.matches_filter(Some(LogLevel::Debug)));

        // No filter shows all
        assert!(info_entry.matches_filter(None));
        assert!(debug_entry.matches_filter(None));
    }

    #[test]
    fn test_log_viewer_filtering() {
        let mut viewer = LogViewer::new();

        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z ERROR test: error".to_string()));
        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z  WARN test: warning".to_string()));
        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z  INFO test: info".to_string()));
        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z DEBUG test: debug".to_string()));

        // No filter - show all
        assert_eq!(viewer.get_filtered_entries().len(), 4);

        // Info filter - show info, warn, error
        viewer.set_filter(Some(LogLevel::Info));
        assert_eq!(viewer.get_filtered_entries().len(), 3);

        // Error filter - show only errors
        viewer.set_filter(Some(LogLevel::Error));
        assert_eq!(viewer.get_filtered_entries().len(), 1);
    }

    #[test]
    fn test_level_counts() {
        let mut viewer = LogViewer::new();

        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z ERROR test: error1".to_string()));
        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z ERROR test: error2".to_string()));
        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z  WARN test: warning".to_string()));
        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z  INFO test: info".to_string()));
        viewer.entries.push(LogEntry::parse("2025-01-22T10:30:45.123456Z DEBUG test: debug".to_string()));

        let (trace, debug, info, warn, error) = viewer.get_level_counts();
        assert_eq!(trace, 0);
        assert_eq!(debug, 1);
        assert_eq!(info, 1);
        assert_eq!(warn, 1);
        assert_eq!(error, 2);
    }
}
