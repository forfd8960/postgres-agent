//! Tool trait definition and core types.
//!
//! This module defines the [`Tool`] trait that all tools must implement,
//! along with supporting types for tool definitions, calls, results, and context.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::ToolError;

/// Tool definition for LLM integration.
///
/// Contains all information the LLM needs to understand and use the tool,
/// including name, description, and JSON Schema for parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// Unique identifier for the tool.
    pub name: String,
    /// Human-readable description of what the tool does.
    pub description: String,
    /// JSON Schema describing the parameters object.
    #[serde(default)]
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    /// Create a new tool definition.
    #[must_use]
    pub fn new(name: String, description: String, parameters: serde_json::Value) -> Self {
        Self {
            name,
            description,
            parameters,
        }
    }
}

/// A tool call made by the agent.
///
/// Represents a single invocation of a tool by the LLM, including
/// the tool name, arguments, and a unique call ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// Tool name to execute.
    pub name: String,
    /// Arguments as a JSON object.
    pub arguments: serde_json::Value,
    /// Unique identifier for tracking this call.
    pub call_id: String,
}

impl ToolCall {
    /// Create a new tool call.
    #[must_use]
    pub fn new(name: String, arguments: serde_json::Value, call_id: String) -> Self {
        Self {
            name,
            arguments,
            call_id,
        }
    }

    /// Create a tool call with an auto-generated ID.
    #[must_use]
    pub fn with_auto_id(name: String, arguments: serde_json::Value) -> Self {
        Self {
            name,
            arguments,
            call_id: format!("call-{}", uuid::Uuid::new_v4().to_string().split_once('-').unwrap_or((&"", &"")).0),
        }
    }
}

/// Result of a tool execution.
///
/// Wraps the outcome of a tool call, including success status,
/// result data, and timing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    /// Original call ID from the tool call.
    pub call_id: String,
    /// Name of the executed tool.
    pub tool: String,
    /// Result data as JSON.
    pub result: serde_json::Value,
    /// Whether execution succeeded.
    pub success: bool,
    /// Error message if execution failed.
    pub error: Option<String>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

impl ToolResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(call_id: String, tool: String, result: serde_json::Value, duration_ms: u64) -> Self {
        Self {
            call_id,
            tool,
            result,
            success: true,
            error: None,
            duration_ms,
        }
    }

    /// Create a failed result.
    #[must_use]
    pub fn failure(call_id: String, tool: String, error: String, duration_ms: u64) -> Self {
        Self {
            call_id,
            tool,
            result: serde_json::Value::Null,
            success: false,
            error: Some(error),
            duration_ms,
        }
    }
}

/// Context provided during tool execution.
///
/// Carries execution parameters like timeouts that apply to
/// all tool invocations.
#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    /// Optional execution timeout override.
    pub timeout: Option<Duration>,
    /// Optional request ID for tracing.
    pub request_id: Option<String>,
}

impl ToolContext {
    /// Create a new context with optional timeout.
    #[must_use]
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout: Some(timeout),
            request_id: None,
        }
    }

    /// Create a new context with request ID.
    #[must_use]
    pub fn with_request_id(request_id: String) -> Self {
        Self {
            timeout: None,
            request_id: Some(request_id),
        }
    }
}

/// Trait for tool implementations.
///
/// All tools must implement this trait to be registered in the
/// [`ToolRegistry`](super::ToolRegistry). The trait provides:
/// - Tool metadata via [`definition()`](Tool::definition)
/// - Parameter validation
/// - Actual execution via [`execute()`](Tool::execute)
///
/// # Example
///
/// ```rust
/// use async_trait::async_trait;
/// use serde_json::Value;
/// use postgres_agent_tools::{Tool, ToolDefinition, ToolContext, ToolError};
///
/// struct MyTool;
///
/// #[async_trait]
/// impl Tool for MyTool {
///     fn definition(&self) -> ToolDefinition {
///         ToolDefinition::new(
///             "my_tool".to_string(),
///             "Does something useful".to_string(),
///             serde_json::json!({
///                 "type": "object",
///                 "properties": {
///                     "input": { "type": "string" }
///                 },
///                 "required": ["input"]
///             })
///         )
///     }
///
///     async fn execute(
///         &self,
///         args: &Value,
///         _ctx: &ToolContext,
///     ) -> Result<Value, ToolError> {
///         let input = args["input"].as_str().unwrap();
///         Ok(serde_json::json!({ "output": input.to_uppercase() }))
///     }
/// }
/// ```
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool definition for LLM consumption.
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool with the given arguments.
    ///
    /// # Arguments
    /// * `args` - JSON object containing tool-specific parameters
    /// * `ctx` - Execution context with timeout and request ID
    ///
    /// # Errors
    /// Returns a [`ToolError`] if execution fails.
    async fn execute(
        &self,
        args: &serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError>;
}
