//! Error types for the PostgreSQL Agent.

use thiserror::Error;

/// Main error type for the application.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Database error: {message}")]
    Database { message: String },

    #[error("LLM error: {message}")]
    Llm { message: String },

    #[error("TUI error: {message}")]
    Tui { message: String },

    #[error("Tool error: {message}")]
    Tool { message: String },

    #[error("IO error: {message}")]
    Io { message: String },

    #[error("Parse error: {message}")]
    Parse { message: String },
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Self::Configuration {
            message: e.to_string(),
        }
    }
}
