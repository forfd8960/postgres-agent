//! Common result types for PostgreSQL Agent.
//!
//! Provides re-exported error types and a generic result type
//! used across the application.

pub use anyhow::{Context as AnyhowContext, Error, Result};
pub use thiserror::Error as ThisError;

/// Application-specific error types.
///
/// This enum provides structured errors for common failure modes
/// that include context for debugging.
#[derive(Debug, ThisError)]
pub enum AppError {
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("LLM error: {message}")]
    Llm {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Parse error: {message}")]
    Parse {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl AppError {
    /// Create a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            source: None,
        }
    }

    /// Create a configuration error with a source.
    #[must_use]
    pub fn config_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Configuration {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a database error.
    #[must_use]
    pub fn db(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
            source: None,
        }
    }

    /// Create an LLM error.
    #[must_use]
    pub fn llm(message: impl Into<String>) -> Self {
        Self::Llm {
            message: message.into(),
            source: None,
        }
    }
}
