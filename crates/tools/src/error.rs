//! Tool errors.

use thiserror::Error;

/// Errors from tool execution.
#[derive(Debug, Error)]
pub enum ToolError {
    #[error("Tool not found: {tool_name}")]
    NotFound { tool_name: String },

    #[error("Tool execution failed: {reason}")]
    ExecutionFailed { reason: String },

    #[error("Tool execution timed out")]
    Timeout,

    #[error("Permission denied for tool: {tool_name}")]
    PermissionDenied { tool_name: String },
}
