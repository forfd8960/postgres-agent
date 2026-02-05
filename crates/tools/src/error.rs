//! Tool errors.

use thiserror::Error;
use postgres_agent_db::DbError;

/// Errors from tool execution.
#[derive(Debug, Error)]
pub enum ToolError {
    /// Tool not found in registry.
    #[error("Tool not found: {tool_name}")]
    NotFound {
        /// Name of the missing tool.
        tool_name: String,
    },

    /// Tool execution failed.
    #[error("Tool execution failed: {reason}")]
    ExecutionFailed {
        /// Details about why execution failed.
        reason: String,
    },

    /// Tool execution timed out.
    #[error("Tool execution timed out")]
    Timeout,

    /// Permission denied for tool.
    #[error("Permission denied for tool: {tool_name}")]
    PermissionDenied {
        /// Name of the restricted tool.
        tool_name: String,
    },

    /// Invalid arguments provided to tool.
    #[error("Invalid arguments for tool {tool_name}: {details}")]
    InvalidArguments {
        /// Name of the tool.
        tool_name: String,
        /// Details about the invalid arguments.
        details: String,
    },

    /// Database error during tool execution.
    #[error("Database error: {source}")]
    Database {
        /// Underlying database error.
        #[from]
        source: DbError,
    },

    /// Safety validation failed.
    #[error("Safety validation failed: {reason}")]
    SafetyViolation {
        /// Reason for the safety violation.
        reason: String,
    },
}

impl From<serde_json::Error> for ToolError {
    fn from(e: serde_json::Error) -> Self {
        Self::ExecutionFailed {
            reason: format!("JSON serialization error: {}", e),
        }
    }
}
