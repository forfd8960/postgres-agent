//! Core agent errors.

use thiserror::Error;

/// Errors that can occur during agent execution.
#[derive(Debug, Error)]
pub enum AgentError {
    /// Maximum iterations exceeded for query.
    #[error("Maximum iterations ({iterations}) exceeded for query")]
    MaxIterationsExceeded { iterations: u32 },

    /// Invalid tool call.
    #[error("Invalid tool call: {details}")]
    InvalidToolCall { details: String },

    /// Tool execution failed.
    #[error("Tool execution failed: {tool_name} - {reason}")]
    ToolExecutionFailed { tool_name: String, reason: String },

    /// Context exceeds model limit.
    #[error("Context exceeds model limit: {size} tokens (limit: {limit})")]
    ContextTooLarge { size: usize, limit: usize },

    /// LLM API error.
    #[error("LLM error: {message}")]
    LlmError { message: String },

    /// Database error.
    #[error("Database error: {message}")]
    DatabaseError { message: String },

    /// Safety violation.
    #[error("Safety violation: {reason}")]
    SafetyViolation { reason: String },

    /// Configuration error.
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Tool not found.
    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    /// Timeout error.
    #[error("Operation timed out after {seconds}s")]
    Timeout { seconds: u64 },

    /// Invalid state for operation.
    #[error("Invalid agent state: {state}")]
    InvalidState { state: String },

    /// Conversation history error.
    #[error("History error: {message}")]
    HistoryError { message: String },

    /// Serialization error.
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
}

impl AgentError {
    /// Create a new tool not found error.
    #[must_use]
    pub fn tool_not_found(name: impl Into<String>) -> Self {
        AgentError::ToolNotFound { name: name.into() }
    }

    /// Create a new LLM error.
    #[must_use]
    pub fn llm_error(message: impl Into<String>) -> Self {
        AgentError::LlmError { message: message.into() }
    }

    /// Create a new database error.
    #[must_use]
    pub fn database_error(message: impl Into<String>) -> Self {
        AgentError::DatabaseError { message: message.into() }
    }

    /// Create a new safety violation error.
    #[must_use]
    pub fn safety_violation(message: impl Into<String>) -> Self {
        AgentError::SafetyViolation { reason: message.into() }
    }

    /// Create a new timeout error.
    #[must_use]
    pub fn timeout(seconds: u64) -> Self {
        AgentError::Timeout { seconds }
    }

    /// Check if this is a retryable error.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            AgentError::LlmError { .. } => true,
            AgentError::Timeout { .. } => true,
            AgentError::DatabaseError { .. } => true,
            _ => false,
        }
    }

    /// Get a user-friendly error message.
    #[must_use]
    pub fn user_message(&self) -> String {
        match self {
            AgentError::MaxIterationsExceeded { .. } => {
                "The query is too complex and requires too many reasoning steps.".to_string()
            }
            AgentError::InvalidToolCall { details } => {
                format!("Invalid tool call: {}", details)
            }
            AgentError::ToolExecutionFailed { tool_name, reason } => {
                format!("Tool '{}' failed: {}", tool_name, reason)
            }
            AgentError::ContextTooLarge { size, limit } => {
                format!("Context too large ({} > {} tokens)", size, limit)
            }
            AgentError::LlmError { message } => {
                format!("AI model error: {}", message)
            }
            AgentError::DatabaseError { message } => {
                format!("Database error: {}", message)
            }
            AgentError::SafetyViolation { reason } => {
                format!("Query blocked for safety: {}", reason)
            }
            AgentError::ConfigurationError { message } => {
                format!("Configuration error: {}", message)
            }
            AgentError::ToolNotFound { name } => {
                format!("Unknown tool: {}", name)
            }
            AgentError::Timeout { seconds } => {
                format!("Operation timed out after {} seconds", seconds)
            }
            AgentError::InvalidState { state } => {
                format!("Invalid agent state: {}", state)
            }
            AgentError::HistoryError { message } => {
                format!("History error: {}", message)
            }
            AgentError::SerializationError { message } => {
                format!("Serialization error: {}", message)
            }
        }
    }
}

/// Result type for agent operations.
pub type AgentResult<T> = Result<T, AgentError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let error = AgentError::tool_not_found("test_tool");
        assert_eq!(error.to_string(), "Tool not found: test_tool");
    }

    #[test]
    fn test_user_message() {
        let error = AgentError::max_iterations_exceeded(10);
        assert!(error.user_message().contains("too complex"));
    }

    #[test]
    fn test_retryable() {
        assert!(AgentError::llm_error("test").is_retryable());
        assert!(AgentError::timeout(30).is_retryable());
        assert!(!AgentError::tool_not_found("test").is_retryable());
    }
}
