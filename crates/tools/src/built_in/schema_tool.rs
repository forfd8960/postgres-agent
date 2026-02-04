//! Schema introspection tool.

use serde::Deserialize;

use crate::{Tool, ToolContext, ToolDefinition};

/// Arguments for the schema tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaToolArgs {
    /// Optional table name filter.
    table_filter: Option<String>,
}

/// Schema introspection tool (stub).
#[derive(Debug)]
pub struct SchemaTool;

impl SchemaTool {
    /// Create a new schema tool.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for SchemaTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Tool for SchemaTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "get_schema".to_string(),
            description: "Get database schema information".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "table_filter": {
                        "type": "string",
                        "description": "Optional table name filter"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<serde_json::Value, crate::ToolError> {
        let _args: SchemaToolArgs = serde_json::from_value(args.clone())
            .map_err(|e| crate::ToolError::ExecutionFailed {
                reason: format!("Invalid arguments: {}", e),
            })?;

        // Stub implementation
        Ok(serde_json::json!({
            "status": "not_implemented",
            "tables": []
        }))
    }
}
