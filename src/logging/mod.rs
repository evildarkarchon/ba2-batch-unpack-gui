//! Logging configuration for Unpackrr-rs
//!
//! This module provides comprehensive logging setup using the `tracing` ecosystem.
//! Features:
//! - Console output with color formatting
//! - File output with daily rotation
//! - Configurable log levels
//! - Environment variable override (`RUST_LOG`)
//! - Integration with application config

use crate::config::{AppConfig, LogLevel};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Initialize the logging system
///
/// This function sets up logging with both console and file output.
/// Console logs are formatted with color and timestamps.
/// File logs are written to the application's data directory with daily rotation.
///
/// # Arguments
///
/// * `config` - Application configuration (optional, uses defaults if None)
///
/// # Returns
///
/// `Ok(())` on success, error if logging initialization fails
///
/// # Examples
///
/// ```no_run
/// use unpackrr::logging;
/// use unpackrr::config::AppConfig;
///
/// fn main() -> anyhow::Result<()> {
///     let config = AppConfig::load().ok();
///     logging::init(config.as_ref())?;
///     Ok(())
/// }
/// ```
pub fn init(config: Option<&AppConfig>) -> Result<()> {
    // Determine log level from config or default to INFO
    let log_level = config
        .map_or(Level::INFO, |c| config_log_level_to_tracing(c.advanced.log_level));

    // Check if debug mode is enabled
    let show_debug = config.is_some_and(|c| c.advanced.show_debug);

    // Create environment filter
    // Priority: RUST_LOG env var > config setting > default (INFO)
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let level_str = if show_debug {
            "debug"
        } else {
            match log_level {
                Level::ERROR => "error",
                Level::WARN => "warn",
                Level::INFO => "info",
                Level::DEBUG => "debug",
                Level::TRACE => "trace",
            }
        };

        EnvFilter::new(format!("unpackrr={level_str},{level_str}=warn"))
    });

    // Console layer with color and formatting
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(show_debug)
        .with_thread_names(show_debug)
        .with_file(show_debug)
        .with_line_number(show_debug)
        .with_span_events(if show_debug {
            FmtSpan::ENTER | FmtSpan::CLOSE
        } else {
            FmtSpan::NONE
        })
        .with_ansi(true)
        .with_filter(env_filter.clone());

    // File layer with rotation
    let file_layer = create_file_appender()?.map(|file_appender| fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false) // No color codes in file
                .with_writer(file_appender)
                .with_filter(env_filter));

    // Build and initialize the subscriber
    let registry = tracing_subscriber::registry().with(console_layer);

    if let Some(file_layer) = file_layer {
        registry.with(file_layer).try_init()?;
    } else {
        registry.try_init()?;
    }

    Ok(())
}

/// Create a file appender for log rotation
///
/// Logs are written to the application's data directory under a "logs" subdirectory.
/// Files are rotated daily with the naming pattern: `unpackrr-YYYY-MM-DD.log`
fn create_file_appender() -> Result<Option<tracing_appender::non_blocking::NonBlocking>> {
    // Get application data directory
    let project_dirs = ProjectDirs::from("com", "evildarkarchon", "unpackrr")
        .context("Failed to determine application data directory")?;

    let log_dir = project_dirs.data_dir().join("logs");

    // Create logs directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;

    // Create daily rotating file appender
    let file_appender = tracing_appender::rolling::daily(&log_dir, "unpackrr.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Note: We're intentionally leaking guard here because we want logging
    // to persist for the entire application lifetime. In a production app,
    // you'd want to store the guard somewhere (e.g., in main())
    std::mem::forget(guard);

    Ok(Some(non_blocking))
}

/// Get the log directory path
///
/// Returns the directory where log files are stored.
pub fn get_log_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "evildarkarchon", "unpackrr")
        .context("Failed to determine application data directory")?;

    Ok(project_dirs.data_dir().join("logs"))
}

/// Convert config log level to tracing Level
const fn config_log_level_to_tracing(level: LogLevel) -> Level {
    match level {
        LogLevel::Fatal | LogLevel::Error => Level::ERROR,
        LogLevel::Warning => Level::WARN,
        LogLevel::Info => Level::INFO,
        LogLevel::Debug => Level::DEBUG,
        LogLevel::Trace => Level::TRACE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(
            config_log_level_to_tracing(LogLevel::Fatal),
            Level::ERROR
        );
        assert_eq!(
            config_log_level_to_tracing(LogLevel::Error),
            Level::ERROR
        );
        assert_eq!(
            config_log_level_to_tracing(LogLevel::Warning),
            Level::WARN
        );
        assert_eq!(config_log_level_to_tracing(LogLevel::Info), Level::INFO);
        assert_eq!(config_log_level_to_tracing(LogLevel::Debug), Level::DEBUG);
        assert_eq!(config_log_level_to_tracing(LogLevel::Trace), Level::TRACE);
    }

    #[test]
    fn test_get_log_dir() {
        let log_dir = get_log_dir();
        assert!(log_dir.is_ok());

        let path = log_dir.unwrap();
        assert!(path.to_string_lossy().contains("unpackrr"));
        assert!(path.to_string_lossy().contains("logs"));
    }
}
