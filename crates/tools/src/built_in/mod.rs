//! Built-in tools.

use async_trait::async_trait;
use serde::Deserialize;

use crate::{Tool, ToolContext, ToolDefinition};

/// Arguments for the query tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryToolArgs {
    /// The SQL query to execute.
    pub sql: String,
}

/// Arguments for the schema tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaToolArgs {
    /// Optional table name filter.
    pub table_filter: Option<String>,
}

/// All available tool types.
#[derive(Debug)]
pub enum BuiltInTool {
    /// Query execution tool.
    Query,
    /// Schema introspection tool.
    Schema,
}

impl BuiltInTool {
    /// Get the tool name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInTool::Query => "execute_query",
            BuiltInTool::Schema => "get_schema",
        }
    }
}

#[async_trait]
impl Tool for BuiltInTool {
    fn definition(&self) -> ToolDefinition {
        match self {
            BuiltInTool::Query => ToolDefinition {
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
            },
            BuiltInTool::Schema => ToolDefinition {
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
            },
        }
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<serde_json::Value, crate::ToolError> {
        match self {
            BuiltInTool::Query => {
                let args: QueryToolArgs = serde_json::from_value(args.clone())
                    .map_err(|e| crate::ToolError::ExecutionFailed {
                        reason: format!("Invalid arguments: {}", e),
                    })?;
                Ok(serde_json::json!({
                    "status": "not_implemented",
                    "sql": args.sql
                }))
            }
            BuiltInTool::Schema => {
                let _args: SchemaToolArgs = serde_json::from_value(args.clone())
                    .map_err(|e| crate::ToolError::ExecutionFailed {
                        reason: format!("Invalid arguments: {}", e),
                    })?;
                Ok(serde_json::json!({
                    "status": "not_implemented",
                    "tables": []
                }))
            }
        }
    }
}
