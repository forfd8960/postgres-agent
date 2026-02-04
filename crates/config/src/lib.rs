//! Configuration management for PostgreSQL Agent.
//!
//! Provides YAML configuration parsing, environment variable overrides,
//! and configuration validation.

#![warn(missing_docs)]

pub mod app_config;
pub mod database;
pub mod error;
pub mod loader;
pub mod llm;
pub mod safety;

pub use app_config::{AppConfig, Config};
pub use database::DatabaseProfile;
pub use error::ConfigError;
pub use loader::ConfigLoader;
pub use llm::LlmConfig;
pub use safety::SafetyConfig;
