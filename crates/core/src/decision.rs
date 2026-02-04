//! Agent decision types.

use serde::{Deserialize, Serialize};

/// A decision made by the agent after reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentDecision {
    /// Continue reasoning, no tool call needed.
    #[serde(rename = "reasoning")]
    Reasoning {
        /// The reasoning trace.
        thought: String,
    },
    /// Execute a tool call.
    #[serde(rename = "tool_call")]
    ToolCall(ToolCall),
    /// Provide final answer to user.
    #[serde(rename = "final_answer")]
    FinalAnswer(String),
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
