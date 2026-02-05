# Week 5 Implementation Summary: Tool System

## Document Information

| Attribute | Value |
|-----------|-------|
| Version | 1.0.0 |
| Status | Completed |
| Date | 2026-02-05 |
| Week | 5 |

---

## Overview

Week 5 focused on implementing the Tool System for the PostgreSQL Agent, providing a flexible framework for defining and executing tools that the agent can use to interact with the database.

---

## Deliverables

| Task | Status | Description |
|------|--------|-------------|
| 5.1 | Completed | Define Tool trait for extensibility |
| 5.2 | Completed | Implement ToolRegistry for tool management |
| 5.3 | Completed | Implement built-in tools (query, schema, explain) |
| 5.4 | Completed | Implement tool context and execution |
| 5.5 | Completed | Implement parallel execution support |

---

## Files Created/Modified

### `crates/tools/src/lib.rs`

**Updated Module Structure:**
- Added comprehensive crate-level documentation
- Re-exported `create_builtin_tools` helper function
- Re-exported database types for tool convenience

### `crates/tools/src/trait_def.rs`

**Enhanced Tool Trait Types:**
- `ToolDefinition` - LLM tool metadata with JSON Schema parameters
- `ToolCall` - Tool invocation with auto-generated call IDs
- `ToolResult` - Execution outcome with timing and error handling
- `ToolContext` - Execution context with timeout and request ID
- `Tool` trait - Core trait for all tool implementations with documentation

**New Helper Methods:**
- `ToolDefinition::new()` - Constructor
- `ToolCall::new()` and `ToolCall::with_auto_id()` - Constructors with ID generation
- `ToolResult::success()` and `ToolResult::failure()` - Result builders
- `ToolContext::with_timeout()` and `ToolContext::with_request_id()` - Context builders

### `crates/tools/src/error.rs`

**Enhanced Error Types:**
- `ToolError` enumeration with:
  - `NotFound { tool_name }` - Tool not registered
  - `ExecutionFailed { reason }` - General execution failure
  - `Timeout` - Execution timed out
  - `PermissionDenied { tool_name }` - Access denied
  - `InvalidArguments { tool_name, details }` - Argument validation failure
  - `Database { source }` - Wrapped database error
  - `SafetyViolation { reason }` - Safety check failed
- `From<serde_json::Error>` implementation

### `crates/tools/src/registry.rs`

**Tool Registry Implementation:**
- `ToolRegistry` - HashMap-based tool storage
- `register(tool)` - Register a new tool
- `get(name)` - Retrieve tool by name
- `contains(name)` - Check tool existence
- `get_definitions()` - Get all tool definitions for LLM
- `execute(name, args, ctx)` - Execute tool by name

### `crates/tools/src/executor.rs`

**Enhanced Tool Executor:**
- `ToolExecutor` - Main execution engine
- `execute(name, args, ctx)` - Execute single tool with timing
- `execute_with_result(call, ctx)` - Execute with wrapped result
- `execute_parallel(calls, ctx)` - Execute multiple tools concurrently using `futures::join_all`
- `execute_batch(calls, ctx, stop_on_error)` - Sequential batch execution with error handling
- `registry()` - Access underlying registry

### `crates/tools/src/built_in/mod.rs`

**Implemented Built-in Tools:**

| Tool | Description |
|------|-------------|
| `QueryTool` | Execute SELECT queries |
| `SchemaTool` | Introspect database schema |
| `ListTablesTool` | List tables in schema |
| `DescribeTableTool` | Get column details for table |
| `ExplainTool` | Get query execution plan |

**Tool Arguments:**

```rust
// QueryToolArgs
struct QueryToolArgs { sql: String }

// SchemaToolArgs
struct SchemaToolArgs { table_filter: Option<String> }

// ListTablesToolArgs
struct ListTablesToolArgs { schema: Option<String> }

// DescribeTableToolArgs
struct DescribeTableToolArgs { table_name: String }

// ExplainToolArgs
struct ExplainToolArgs { sql: String }
```

**Helper Function:**
- `create_builtin_tools(db: DbConnection) -> Vec<BuiltInTool>` - Creates all built-in tools

### `crates/tools/Cargo.toml`

**Added Dependencies:**
- `futures = "0.3"` - For parallel execution
- `uuid = { version = "1", features = ["v4"] }` - For call ID generation

---

## Architecture

### Tool Execution Flow

```
ToolRegistry
    ↓
ToolExecutor::execute(name, args)
    ↓
ToolRegistry.get(name)
    ↓
Tool::execute(args, context)
    ↓
Database Operations
    ↓
ToolResult { success, result, duration_ms }
```

### Parallel Execution Flow

```
Multiple ToolCalls
    ↓
ToolExecutor::execute_parallel(calls)
    ↓
futures::future::join_all(futures)
    ↓
Vec<ToolResult> (same order as input)
```

### Tool Definition Schema

```json
{
  "name": "execute_query",
  "description": "Execute a SQL SELECT query...",
  "parameters": {
    "type": "object",
    "properties": {
      "sql": {
        "type": "string",
        "description": "The SQL query to execute"
      }
    },
    "required": ["sql"]
  }
}
```

---

## Key Features

### 1. Extensible Tool System

```rust
use postgres_agent_tools::{Tool, ToolDefinition, ToolContext, ToolError};

struct MyCustomTool {
    // Custom state
}

#[async_trait]
impl Tool for MyCustomTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "my_tool".to_string(),
            "Does something useful".to_string(),
            serde_json::json!({ /* parameters */ })
        )
    }

    async fn execute(
        &self,
        args: &serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        // Implementation
        Ok(serde_json::json!({ "result": "..." }))
    }
}
```

### 2. Parallel Tool Execution

```rust
let calls = vec![
    ToolCall::with_auto_id("execute_query".to_string(), serde_json::json!({"sql": "SELECT 1"})),
    ToolCall::with_auto_id("execute_query".to_string(), serde_json::json!({"sql": "SELECT 2"})),
];

let results = executor.execute_parallel(&calls, &ctx).await;
// Results are in the same order as input
```

### 3. Context-based Execution Control

```rust
let ctx = ToolContext::with_timeout(Duration::from_secs(30))
    .with_request_id("request-123".to_string());

let result = executor.execute("execute_query", &args, &ctx).await?;
```

---

## Tool Definitions

### execute_query

```json
{
  "name": "execute_query",
  "description": "Execute a SQL SELECT query and return results in JSON format. Only SELECT queries are allowed.",
  "parameters": {
    "type": "object",
    "properties": {
      "sql": {
        "type": "string",
        "description": "The SQL SELECT query to execute"
      }
    },
    "required": ["sql"]
  }
}
```

### get_schema

```json
{
  "name": "get_schema",
  "description": "Get the complete database schema including all tables and their columns. Optionally filter by table name prefix.",
  "parameters": {
    "type": "object",
    "properties": {
      "tableFilter": {
        "type": "string",
        "description": "Optional table name prefix filter"
      }
    }
  }
}
```

### list_tables

```json
{
  "name": "list_tables",
  "description": "List all table names in a database schema. Defaults to 'public' schema.",
  "parameters": {
    "type": "object",
    "properties": {
      "schema": {
        "type": "string",
        "description": "Schema name (defaults to 'public')"
      }
    }
  }
}
```

### describe_table

```json
{
  "name": "describe_table",
  "description": "Get detailed column information for a specific table. Returns column name, type, nullability, and defaults.",
  "parameters": {
    "type": "object",
    "properties": {
      "tableName": {
        "type": "string",
        "description": "Name of the table to describe"
      }
    },
    "required": ["tableName"]
  }
}
```

### explain_query

```json
{
  "name": "explain_query",
  "description": "Get the query execution plan for a SQL query using EXPLAIN ANALYZE. Shows how the query will be executed.",
  "parameters": {
    "type": "object",
    "properties": {
      "sql": {
        "type": "string",
        "description": "The SQL query to explain"
      }
    },
    "required": ["sql"]
  }
}
```

---

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| tokio | 1.x | Async runtime |
| serde | 1.0 | Serialization |
| serde_json | 1.0 | JSON handling |
| thiserror | 2.0 | Error derivation |
| futures | 0.3 | Parallel execution |
| uuid | 1.x | Call ID generation |
| async-trait | 0.1 | Async trait support |

---

## Usage Example

```rust
use postgres_agent_tools::{ToolRegistry, ToolExecutor, create_builtin_tools};
use postgres_agent_db::DbConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database connection
    let db = DbConnection::from_url("postgres://localhost/mydb").await?;

    // Create registry and register built-in tools
    let mut registry = ToolRegistry::default();
    for tool in create_builtin_tools(db) {
        registry.register(tool);
    }

    // Create executor
    let executor = ToolExecutor::new(registry);

    // Execute a query
    let result = executor.execute(
        "execute_query",
        &serde_json::json!({"sql": "SELECT 1 AS num"}),
        &ToolContext::default()
    ).await?;

    println!("Result: {:?}", result);
    Ok(())
}
```

---

## Next Steps (Week 6)

Week 6 will focus on Safety & Validation:
- Implement SafetyValidator with blacklist pattern matching
- Implement SQL classification (SELECT vs DML/DDL)
- Implement safety levels (ReadOnly, Balanced, Permissive)
- Implement confirmation workflow for risky operations
- Integrate safety checks into tool execution

---

## Notes

- All tools are Send + Sync for thread-safe execution
- Tool trait uses async-trait for stable async fn in traits
- Call IDs are auto-generated for tracing tool invocations
- Execution timing is tracked for all tool calls
- Parallel execution preserves input order using `join_all`
- Built-in tools use database connection cloning (efficient due to pool)
