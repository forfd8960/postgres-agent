//! Configuration errors.

use thiserror::Error;

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    #[error("Failed to parse configuration: {source}")]
    ParseError { source: serde_yaml::Error },

    #[error("Invalid configuration: {message}")]
    Invalid { message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Database profile not found: {name}")]
    ProfileNotFound { name: String },

    #[error("Environment variable error: {source}")]
    EnvVarError { source: std::env::VarError },
}
