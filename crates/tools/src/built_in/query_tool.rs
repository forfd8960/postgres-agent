//! Query execution tool.

use serde::Deserialize;

use crate::{Tool, ToolContext, ToolDefinition};

/// Arguments for the query tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryToolArgs {
    /// The SQL query to execute.
    sql: String,
}

/// Query execution tool (stub).
#[derive(Debug)]
pub struct QueryTool;

impl QueryTool {
    /// Create a new query tool.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for QueryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Tool for QueryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "execute_query".to_string(),
            description: "Execute a SQL SELECT query".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "sql": {
                        "type": "string",
                        "description": "The SQL query to execute"
                    }
                },
                "required": ["sql"]
            }),
        }
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<serde_json::Value, crate::ToolError> {
        let args: QueryToolArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::ToolError::ExecutionFailed {
                reason: format!("Invalid arguments: {}", e),
            })?;

        // Stub implementation
        Ok(serde_json::json!({
            "status": "not_implemented",
            "sql": args.sql
        }))
    }
}
