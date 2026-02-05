//! Tool system for PostgreSQL Agent.
//!
//! This crate provides the tool system for the PostgreSQL Agent, including:
//! - [`Tool`](trait_def::Tool) trait for implementing tools
//! - [`ToolRegistry`](registry::ToolRegistry) for managing tools
//! - [`ToolExecutor`](executor::ToolExecutor) for executing tools
//! - Built-in tools for database operations
//!
//! # Example
//!
//! ```rust
//! use postgres_agent_tools::{ToolRegistry, ToolExecutor, BuiltInTool};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut registry = ToolRegistry::default();
//!
//!     // Register built-in tools
//!     // registry.register(BuiltInTool::Query(...));
//!
//!     let executor = ToolExecutor::new(registry);
//! }
//! ```

#![warn(missing_docs)]

pub mod built_in;
pub mod error;
pub mod executor;
pub mod registry;
pub mod trait_def;

// Re-export types for convenience
pub use built_in::{BuiltInTool, create_builtin_tools};
pub use error::ToolError;
pub use executor::ToolExecutor;
pub use registry::ToolRegistry;
pub use trait_def::{Tool, ToolCall, ToolContext, ToolDefinition, ToolResult};

// Re-export database types for tools
pub use postgres_agent_db::{DbConnection, QueryExecutor};
