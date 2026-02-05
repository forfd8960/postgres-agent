//! Built-in tools for PostgreSQL Agent.
//!
//! This module provides the core database tools that the agent uses
//! to interact with PostgreSQL databases.

use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

use crate::trait_def::{Tool, ToolContext, ToolDefinition};
use crate::{ToolError, DbConnection, QueryExecutor};

/// Arguments for the query execution tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryToolArgs {
    /// The SQL query to execute.
    pub sql: String,
}

/// Arguments for the schema introspection tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaToolArgs {
    /// Optional table name filter.
    #[serde(default)]
    pub table_filter: Option<String>,
}

/// Arguments for the list tables tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListTablesToolArgs {
    /// Optional schema name (defaults to 'public').
    #[serde(default)]
    pub schema: Option<String>,
}

/// Arguments for the describe table tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescribeTableToolArgs {
    /// Name of the table to describe.
    pub table_name: String,
}

/// Arguments for the explain query tool.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplainToolArgs {
    /// The SQL query to explain.
    pub sql: String,
}

/// All available tool types.
///
/// This enum wraps all built-in tools and provides a unified interface
/// for tool execution and registration.
#[derive(Debug)]
pub enum BuiltInTool {
    /// Query execution tool.
    Query(QueryTool),
    /// Schema introspection tool.
    Schema(SchemaTool),
    /// List tables tool.
    ListTables(ListTablesTool),
    /// Describe table tool.
    DescribeTable(DescribeTableTool),
    /// Explain query tool.
    Explain(ExplainTool),
}

impl BuiltInTool {
    /// Get the tool name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            BuiltInTool::Query(_) => "execute_query",
            BuiltInTool::Schema(_) => "get_schema",
            BuiltInTool::ListTables(_) => "list_tables",
            BuiltInTool::DescribeTable(_) => "describe_table",
            BuiltInTool::Explain(_) => "explain_query",
        }
    }
}

/// Query execution tool.
///
/// Executes SELECT queries against the database and returns results
/// in JSON format.
#[derive(Debug)]
pub struct QueryTool {
    /// Database connection.
    db: DbConnection,
}

impl QueryTool {
    /// Create a new query tool.
    #[must_use]
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Tool for QueryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "execute_query".to_string(),
            description: "Execute a SQL SELECT query and return results in JSON format. Only SELECT queries are allowed.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "sql": {
                        "type": "string",
                        "description": "The SQL SELECT query to execute"
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
    ) -> Result<serde_json::Value, ToolError> {
        let args: QueryToolArgs = serde_json::from_value(args.clone())
            .map_err(|e| ToolError::InvalidArguments {
                tool_name: "execute_query".to_string(),
                details: format!("Invalid arguments: {}", e),
            })?;

        debug!("Executing query: {}", args.sql);

        let executor = QueryExecutor::new(self.db.clone());
        let result = executor.execute_query(&args.sql).await?;

        Ok(serde_json::json!({
            "columns": result.columns,
            "rows": result.rows,
            "rowCount": result.row_count,
            "truncated": result.truncated,
            "executionTimeMs": result.execution_time_ms
        }))
    }
}

/// Schema introspection tool.
///
/// Retrieves the database schema including all tables and their columns.
#[derive(Debug)]
pub struct SchemaTool {
    /// Database connection.
    db: DbConnection,
}

impl SchemaTool {
    /// Create a new schema tool.
    #[must_use]
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Tool for SchemaTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "get_schema".to_string(),
            description: "Get the complete database schema including all tables and their columns. Optionally filter by table name prefix.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "tableFilter": {
                        "type": "string",
                        "description": "Optional table name prefix filter"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        let args: SchemaToolArgs = serde_json::from_value(args.clone())
            .map_err(|e| ToolError::InvalidArguments {
                tool_name: "get_schema".to_string(),
                details: format!("Invalid arguments: {}", e),
            })?;

        debug!("Getting schema with filter: {:?}", args.table_filter);

        let executor = QueryExecutor::new(self.db.clone());
        let schema = executor.get_schema(args.table_filter.as_deref()).await?;

        Ok(serde_json::json!({
            "tables": schema.tables,
            "columns": schema.columns
        }))
    }
}

/// List tables tool.
///
/// Lists all table names in a database schema.
#[derive(Debug)]
pub struct ListTablesTool {
    /// Database connection.
    db: DbConnection,
}

impl ListTablesTool {
    /// Create a new list tables tool.
    #[must_use]
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Tool for ListTablesTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "list_tables".to_string(),
            description: "List all table names in a database schema. Defaults to 'public' schema.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "schema": {
                        "type": "string",
                        "description": "Schema name (defaults to 'public')"
                    }
                }
            }),
        }
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        let args: ListTablesToolArgs = serde_json::from_value(args.clone())
            .map_err(|e| ToolError::InvalidArguments {
                tool_name: "list_tables".to_string(),
                details: format!("Invalid arguments: {}", e),
            })?;

        debug!("Listing tables in schema: {:?}", args.schema);

        let executor = QueryExecutor::new(self.db.clone());
        let tables = executor.list_tables(args.schema.as_deref()).await?;

        Ok(serde_json::json!({
            "tables": tables
        }))
    }
}

/// Describe table tool.
///
/// Returns detailed column information for a specific table.
#[derive(Debug)]
pub struct DescribeTableTool {
    /// Database connection.
    db: DbConnection,
}

impl DescribeTableTool {
    /// Create a new describe table tool.
    #[must_use]
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Tool for DescribeTableTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "describe_table".to_string(),
            description: "Get detailed column information for a specific table. Returns column name, type, nullability, and defaults.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "tableName": {
                        "type": "string",
                        "description": "Name of the table to describe"
                    }
                },
                "required": ["tableName"]
            }),
        }
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        let args: DescribeTableToolArgs = serde_json::from_value(args.clone())
            .map_err(|e| ToolError::InvalidArguments {
                tool_name: "describe_table".to_string(),
                details: format!("Invalid arguments: {}", e),
            })?;

        debug!("Describing table: {}", args.table_name);

        let executor = QueryExecutor::new(self.db.clone());
        let columns = executor.describe_table(&args.table_name).await?;

        Ok(serde_json::json!({
            "tableName": args.table_name,
            "columns": columns
        }))
    }
}

/// Explain query tool.
///
/// Returns the query execution plan for a SQL query.
#[derive(Debug)]
pub struct ExplainTool {
    /// Database connection.
    db: DbConnection,
}

impl ExplainTool {
    /// Create a new explain tool.
    #[must_use]
    pub fn new(db: DbConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Tool for ExplainTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "explain_query".to_string(),
            description: "Get the query execution plan for a SQL query using EXPLAIN ANALYZE. Shows how the query will be executed.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "sql": {
                        "type": "string",
                        "description": "The SQL query to explain"
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
    ) -> Result<serde_json::Value, ToolError> {
        let args: ExplainToolArgs = serde_json::from_value(args.clone())
            .map_err(|e| ToolError::InvalidArguments {
                tool_name: "explain_query".to_string(),
                details: format!("Invalid arguments: {}", e),
            })?;

        debug!("Explaining query: {}", args.sql);

        // Wrap query in EXPLAIN ANALYZE
        let explain_sql = format!("EXPLAIN (FORMAT JSON) {}", args.sql);

        let executor = QueryExecutor::new(self.db.clone());
        let result = executor.execute_query(&explain_sql).await?;

        Ok(serde_json::json!({
            "plan": result.rows,
            "rowCount": result.row_count
        }))
    }
}

#[async_trait]
impl Tool for BuiltInTool {
    fn definition(&self) -> ToolDefinition {
        match self {
            BuiltInTool::Query(tool) => tool.definition(),
            BuiltInTool::Schema(tool) => tool.definition(),
            BuiltInTool::ListTables(tool) => tool.definition(),
            BuiltInTool::DescribeTable(tool) => tool.definition(),
            BuiltInTool::Explain(tool) => tool.definition(),
        }
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        match self {
            BuiltInTool::Query(tool) => tool.execute(args, ctx).await,
            BuiltInTool::Schema(tool) => tool.execute(args, ctx).await,
            BuiltInTool::ListTables(tool) => tool.execute(args, ctx).await,
            BuiltInTool::DescribeTable(tool) => tool.execute(args, ctx).await,
            BuiltInTool::Explain(tool) => tool.execute(args, ctx).await,
        }
    }
}

/// Helper function to create all built-in tools from a database connection.
#[must_use]
pub fn create_builtin_tools(db: DbConnection) -> Vec<BuiltInTool> {
    vec![
        BuiltInTool::Query(QueryTool::new(db.clone())),
        BuiltInTool::Schema(SchemaTool::new(db.clone())),
        BuiltInTool::ListTables(ListTablesTool::new(db.clone())),
        BuiltInTool::DescribeTable(DescribeTableTool::new(db.clone())),
        BuiltInTool::Explain(ExplainTool::new(db)),
    ]
}
