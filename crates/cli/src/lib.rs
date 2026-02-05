//! CLI interface for PostgreSQL Agent.
//!
//! Provides clap-based command-line argument parsing and CLI command implementations.

#![warn(missing_docs)]

pub mod args;
pub mod commands;

pub use args::{CliArgs, Commands};
pub use commands::{OutputFormat, QueryContext, QueryResult};
