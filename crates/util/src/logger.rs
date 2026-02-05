//! Logging infrastructure for PostgreSQL Agent.
//!
//! Provides structured logging using `tracing` with sensible defaults
//! for development and production environments.

use std::path::PathBuf;
use std::sync::OnceLock;

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::{self, time::UtcTime};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

/// Global logger guard for non-blocking appender
static LOGGER_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

/// Log file configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Custom log file path (None = stdout only)
    pub log_file: Option<PathBuf>,
    /// Whether to enable JSON logging for production
    pub json_format: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            log_file: None,
            json_format: false,
        }
    }
}

/// Set up logging with the given configuration.
///
/// This function initializes the global tracing subscriber with sensible
/// defaults and optional file logging.
///
/// # Errors
/// Returns an error if the log file cannot be created or written to.
#[tracing::instrument(skip_all)]
pub fn setup_logger(config: &LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let level = parse_log_level(&config.level)?;

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level.to_string()));

    // Determine output format
    let json_format = config.json_format
        || std::env::var("PG_AGENT_LOG_JSON")
            .is_ok_and(|v| v.to_lowercase() == "true");

    if let Some(ref log_path) = config.log_file {
        // Ensure the parent directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Create a file appender
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)?;

        let (non_blocking, guard) = tracing_appender::non_blocking(file);

        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_writer(non_blocking)
                    .with_timer(UtcTime::rfc_3339()),
            );

        tracing::subscriber::set_global_default(subscriber)?;

        LOGGER_GUARD.set(guard).ok();
    } else {
        // stdout-only logging
        let subscriber = tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_timer(UtcTime::rfc_3339()),
            );

        tracing::subscriber::set_global_default(subscriber)?;
    };

    // Log startup message
    tracing::info!(
        level = %config.level,
        json = json_format,
        "Logger initialized"
    );

    Ok(())
}

/// Parse a log level string into a tracing Level.
fn parse_log_level(level: &str) -> Result<Level, LogLevelError> {
    match level.to_lowercase().as_str() {
        "trace" => Ok(Level::TRACE),
        "debug" => Ok(Level::DEBUG),
        "info" => Ok(Level::INFO),
        "warn" | "warning" => Ok(Level::WARN),
        "error" => Ok(Level::ERROR),
        _ => Err(LogLevelError::UnknownLevel {
            level: level.to_string(),
            valid: &["trace", "debug", "info", "warn", "error"],
        }),
    }
}

/// Error type for log level parsing.
#[derive(Debug, thiserror::Error)]
pub enum LogLevelError {
    #[error("Unknown log level: {level}. Valid levels are: {valid:?}")]
    UnknownLevel {
        level: String,
        valid: &'static [&'static str],
    },
}

/// Initialize logging from environment variables.
///
/// Reads configuration from:
/// - `PG_AGENT_LOG_LEVEL`: Log level (default: info)
/// - `PG_AGENT_LOG_JSON`: Enable JSON format (default: false)
/// - `PG_AGENT_LOG_FILE`: Log to file path
#[tracing::instrument(skip_all)]
pub fn setup_logger_from_env() -> Result<(), Box<dyn std::error::Error>> {
    let level = std::env::var("PG_AGENT_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let json_format = std::env::var("PG_AGENT_LOG_JSON")
        .is_ok_and(|v| v.to_lowercase() == "true");
    let log_file = std::env::var("PG_AGENT_LOG_FILE").ok().map(PathBuf::from);

    let config = LogConfig {
        level,
        log_file,
        json_format,
    };

    setup_logger(&config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    #[test]
    fn test_parse_log_level() {
        assert_eq!(parse_log_level("trace").unwrap(), Level::TRACE);
        assert_eq!(parse_log_level("debug").unwrap(), Level::DEBUG);
        assert_eq!(parse_log_level("info").unwrap(), Level::INFO);
        assert_eq!(parse_log_level("warn").unwrap(), Level::WARN);
        assert_eq!(parse_log_level("error").unwrap(), Level::ERROR);
        assert_eq!(parse_log_level("warning").unwrap(), Level::WARN);

        assert!(parse_log_level("invalid").is_err());
    }

    #[test]
    fn test_log_config_defaults() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert!(config.log_file.is_none());
        assert!(!config.json_format);
    }
}
