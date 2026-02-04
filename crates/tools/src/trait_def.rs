//! Tool trait definition.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::ToolError;

/// Tool definition for LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// Unique identifier for the tool.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for parameters.
    pub parameters: serde_json::Value,
}

/// A tool call made by the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// Tool name.
    pub name: String,
    /// Tool arguments (JSON).
    pub arguments: serde_json::Value,
    /// Call ID for tracking.
    pub call_id: String,
}

/// Result of a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    /// Original call ID.
    pub call_id: String,
    /// Tool name.
    pub tool: String,
    /// Result data (JSON).
    pub result: serde_json::Value,
    /// Whether execution was successful.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Context provided during tool execution.
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Execution timeout override.
    pub timeout: Option<Duration>,
}

/// Trait for tool implementations.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get tool definition.
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool with given arguments.
    async fn execute(
        &self,
        args: &serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError>;
}
