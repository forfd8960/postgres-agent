//! Tool registry.

use std::collections::HashMap;

use crate::trait_def::{Tool, ToolDefinition};
use crate::BuiltInTool;
use crate::ToolError;

/// Registry of available tools for the agent.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    /// Registered tools by name.
    tools: HashMap<String, BuiltInTool>,
}

impl ToolRegistry {
    /// Register a new tool.
    pub fn register(&mut self, tool: BuiltInTool) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Get a tool by name.
    pub fn get(&self, name: &str) -> Option<&BuiltInTool> {
        self.tools.get(name)
    }

    /// Check if a tool exists.
    pub fn contains(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get all tool definitions.
    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Execute a tool by name.
    pub async fn execute(
        &self,
        name: &str,
        args: &serde_json::Value,
        ctx: &crate::ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound {
                tool_name: name.to_string(),
            })?;

        tool.execute(args, ctx).await
    }
}
