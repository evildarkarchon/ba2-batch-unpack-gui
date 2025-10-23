//! Retry logic for transient failures (Phase 2.8)
//!
//! Provides utilities for retrying operations that might fail due to
//! temporary issues like file locks, network timeouts, or busy resources.

use crate::error::Error;
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not including the initial attempt)
    pub max_attempts: usize,
    /// Initial delay before the first retry
    pub initial_delay: Duration,
    /// Multiplier for exponential backoff (1.0 = no backoff, 2.0 = double each time)
    pub backoff_multiplier: f64,
    /// Maximum delay between retries
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(5),
        }
    }
}

impl RetryConfig {
    /// Create a configuration for quick retries (short delays, few attempts)
    #[must_use]
    pub fn quick() -> Self {
        Self {
            max_attempts: 2,
            initial_delay: Duration::from_millis(50),
            backoff_multiplier: 1.5,
            max_delay: Duration::from_secs(1),
        }
    }

    /// Create a configuration for persistent retries (long delays, many attempts)
    #[must_use]
    pub fn persistent() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(200),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(10),
        }
    }
}

/// Retry a fallible operation with exponential backoff
///
/// This function will retry the operation if it fails with a transient error.
/// Non-transient errors are returned immediately without retrying.
///
/// # Arguments
///
/// * `config` - Retry configuration (attempts, delays, backoff)
/// * `operation` - The operation to retry (must be `FnMut`)
///
/// # Returns
///
/// The result of the operation, or the last error if all retries failed.
///
/// # Examples
///
/// ```no_run
/// use unpackrr::operations::retry::{retry_with_config, RetryConfig};
/// use std::fs::File;
///
/// let config = RetryConfig::default();
/// let result = retry_with_config(config, || {
///     File::open("/path/to/file.txt")
///         .map_err(|e| e.into())
/// });
/// ```
pub fn retry_with_config<F, T>(config: RetryConfig, mut operation: F) -> Result<T, Error>
where
    F: FnMut() -> Result<T, Error>,
{
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;

                // Check if we should retry
                if !e.is_transient() || attempts > config.max_attempts {
                    tracing::debug!(
                        "Operation failed after {} attempts: {}",
                        attempts,
                        e
                    );
                    return Err(e);
                }

                // Log the retry
                tracing::warn!(
                    "Transient error detected (attempt {}/{}): {}. Retrying in {:?}...",
                    attempts,
                    config.max_attempts,
                    e,
                    delay
                );

                // Wait before retrying
                std::thread::sleep(delay);

                // Calculate next delay with exponential backoff
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.backoff_multiplier).min(config.max_delay.as_secs_f64())
                );
            }
        }
    }
}

/// Retry an operation with default configuration
///
/// Convenience function that uses `RetryConfig::default()`.
///
/// # Examples
///
/// ```no_run
/// use unpackrr::operations::retry::retry;
/// use std::fs;
///
/// let result = retry(|| {
///     fs::remove_file("/path/to/file.txt")
///         .map_err(|e| e.into())
/// });
/// ```
pub fn retry<F, T>(operation: F) -> Result<T, Error>
where
    F: FnMut() -> Result<T, Error>,
{
    retry_with_config(RetryConfig::default(), operation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_retry_succeeds_on_first_attempt() {
        let result = retry(|| Ok(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_retry_succeeds_after_transient_failures() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = retry_with_config(RetryConfig::quick(), move || {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                // Fail with transient error first two times
                Err(Error::IO(std::io::Error::from(std::io::ErrorKind::Interrupted)))
            } else {
                Ok(42)
            }
        });

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3); // 2 failures + 1 success
    }

    #[test]
    fn test_retry_fails_on_permanent_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let result: Result<i32, Error> = retry(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err(Error::other("permanent error"))
        });

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // No retries for permanent errors
    }

    #[test]
    fn test_retry_exhausts_attempts() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(10),
            backoff_multiplier: 1.0,
            max_delay: Duration::from_millis(10),
        };

        let result: Result<i32, Error> = retry_with_config(config, move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err(Error::IO(std::io::Error::from(std::io::ErrorKind::Interrupted)))
        });

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3); // 1 initial + 2 retries
    }
}
