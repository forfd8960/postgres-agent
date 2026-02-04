//! Core agent errors.

use thiserror::Error;

/// Errors that can occur during agent execution.
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Maximum iterations ({iterations}) exceeded for query")]
    MaxIterationsExceeded { iterations: u32 },

    #[error("Invalid tool call: {details}")]
    InvalidToolCall { details: String },

    #[error("Tool execution failed: {tool_name} - {reason}")]
    ToolExecutionFailed { tool_name: String, reason: String },

    #[error("Context exceeds model limit: {size} tokens (limit: {limit})")]
    ContextTooLarge { size: usize, limit: usize },
}
