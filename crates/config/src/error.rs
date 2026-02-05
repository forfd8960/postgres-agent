//! Configuration errors.

use thiserror::Error;

/// Configuration errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file does not exist at the specified path.
    #[error("Configuration file not found: {path}")]
    FileNotFound {
        /// Path to the configuration file.
        path: String,
    },

    /// Failed to parse the configuration file as valid YAML.
    #[error("Failed to parse configuration: {source}")]
    ParseError {
        /// YAML parsing error details.
        source: serde_yaml::Error,
    },

    /// Configuration is invalid (e.g., missing required fields, invalid values).
    #[error("Invalid configuration: {message}")]
    Invalid {
        /// Details about the invalid configuration.
        message: String,
    },

    /// A required configuration field is missing.
    #[error("Missing required field: {field}")]
    MissingField {
        /// Name of the missing field.
        field: String,
    },

    /// The specified database profile was not found.
    #[error("Database profile not found: {name}")]
    ProfileNotFound {
        /// Name of the profile that was not found.
        name: String,
    },

    /// Failed to read an environment variable.
    #[error("Environment variable error: {source}")]
    EnvVarError {
        /// Environment variable error details.
        source: std::env::VarError,
    },

    /// The configuration failed validation.
    #[error("Validation error: {message}")]
    ValidationError {
        /// Details about the validation failure.
        message: String,
    },
}
