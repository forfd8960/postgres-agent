//! Tool system for PostgreSQL Agent.

#![warn(missing_docs)]

pub mod built_in;
pub mod error;
pub mod executor;
pub mod registry;
pub mod trait_def;

pub use built_in::BuiltInTool;
pub use error::ToolError;
pub use executor::ToolExecutor;
pub use registry::ToolRegistry;
pub use trait_def::{Tool, ToolCall, ToolContext, ToolDefinition, ToolResult};
