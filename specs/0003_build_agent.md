# Build Agent with Rust

## Crates

- async-openai for OpenAI API integration
- tokio for async runtime
- serde_json for handling JSON responses
- rig - a Rust framework specifically for building LLM-powered applications

## Modules

Tool/function calling logic
Agent loop (reasoning → action → observation)
Memory/context management
Integration with external services and databases
Prompt templating and management


Great project! Let me break down how to build each module for your PostgreSQL AI agent in Rust.

## 1. Tool/Function Calling Logic

This is where you define the actions your agent can take:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Tool {
    QueryDatabase { sql: String },
    ExportData { query: String, format: String },
    GenerateMigration { description: String },
    RunDML { statement: String },
    GetSchema { table_name: Option<String> },
}

#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

// Define tools for the LLM to understand
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "query_database".to_string(),
            description: "Execute a SELECT query and return results".to_string(),
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
        },
        ToolDefinition {
            name: "run_dml".to_string(),
            description: "Execute INSERT, UPDATE, or DELETE statements".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "statement": {
                        "type": "string",
                        "description": "The DML statement to execute"
                    }
                },
                "required": ["statement"]
            }),
        },
        // Add more tools...
    ]
}

// Execute the tool
pub async fn execute_tool(
    tool: Tool,
    db_pool: &sqlx::PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    match tool {
        Tool::QueryDatabase { sql } => {
            let rows = sqlx::query(&sql)
                .fetch_all(db_pool)
                .await?;
            Ok(format!("Query returned {} rows: {:?}", rows.len(), rows))
        }
        Tool::RunDML { statement } => {
            let result = sqlx::query(&statement)
                .execute(db_pool)
                .await?;
            Ok(format!("Affected {} rows", result.rows_affected()))
        }
        // Implement other tools...
        _ => Ok("Tool not implemented".to_string()),
    }
}
```

## 2. Agent Loop (Reasoning → Action → Observation)

The core loop that drives the agent:

```rust
use reqwest::Client;

pub struct Agent {
    http_client: Client,
    api_key: String,
    db_pool: sqlx::PgPool,
    max_iterations: usize,
}

#[derive(Debug)]
pub struct AgentStep {
    pub reasoning: String,
    pub action: Option<Tool>,
    pub observation: Option<String>,
}

impl Agent {
    pub async fn run(
        &self,
        user_query: &str,
        context: &mut AgentContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut iterations = 0;
        
        while iterations < self.max_iterations {
            iterations += 1;
            
            // 1. REASONING: Ask LLM what to do next
            let prompt = self.build_prompt(user_query, context);
            let llm_response = self.call_llm(&prompt).await?;
            
            // 2. ACTION: Parse and execute tool if LLM wants to use one
            if let Some(tool_call) = self.parse_tool_call(&llm_response)? {
                let observation = execute_tool(tool_call.clone(), &self.db_pool).await?;
                
                // 3. OBSERVATION: Store result and continue loop
                context.add_step(AgentStep {
                    reasoning: llm_response.clone(),
                    action: Some(tool_call),
                    observation: Some(observation.clone()),
                });
                
                // Add observation to context for next iteration
                context.add_message("tool", &observation);
            } else {
                // LLM provided final answer
                return Ok(llm_response);
            }
        }
        
        Err("Max iterations reached".into())
    }
    
    async fn call_llm(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": "claude-sonnet-4-20250514",
                "max_tokens": 4096,
                "messages": [{"role": "user", "content": prompt}],
                "tools": get_tool_definitions(),
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        // Parse response and extract text or tool calls
        Ok(response["content"][0]["text"].as_str().unwrap_or("").to_string())
    }
}
```

## 3. Memory/Context Management

Track conversation history and agent steps:

```rust
use std::collections::VecDeque;

pub struct AgentContext {
    pub messages: VecDeque<Message>,
    pub steps: Vec<AgentStep>,
    pub database_schema: Option<String>,
    max_messages: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub role: String,  // "user", "assistant", "tool"
    pub content: String,
}

impl AgentContext {
    pub fn new(max_messages: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            steps: Vec::new(),
            database_schema: None,
            max_messages,
        }
    }
    
    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push_back(Message {
            role: role.to_string(),
            content: content.to_string(),
        });
        
        // Keep only recent messages to manage context window
        if self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
    }
    
    pub fn add_step(&mut self, step: AgentStep) {
        self.steps.push(step);
    }
    
    pub fn get_conversation_history(&self) -> String {
        self.messages
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
    
    // Cache expensive operations
    pub async fn get_or_fetch_schema(
        &mut self,
        db_pool: &sqlx::PgPool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(schema) = &self.database_schema {
            return Ok(schema.clone());
        }
        
        let schema = fetch_database_schema(db_pool).await?;
        self.database_schema = Some(schema.clone());
        Ok(schema)
    }
}

async fn fetch_database_schema(
    db_pool: &sqlx::PgPool,
) -> Result<String, Box<dyn std::error::Error>> {
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT table_name FROM information_schema.tables 
         WHERE table_schema = 'public'"
    )
    .fetch_all(db_pool)
    .await?;
    
    // Format schema info
    Ok(format!("Tables: {:?}", tables))
}
```

## 4. Integration with External Services and Databases

Set up PostgreSQL connection and other integrations:

```rust
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

pub async fn create_db_pool(
    config: DatabaseConfig,
) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.url)
        .await
}

// Safe query execution with validation
pub async fn safe_query_execution(
    sql: &str,
    db_pool: &sqlx::PgPool,
    allow_mutations: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    // Validate query type
    let sql_upper = sql.trim().to_uppercase();
    let is_select = sql_upper.starts_with("SELECT");
    let is_mutation = sql_upper.starts_with("INSERT") 
        || sql_upper.starts_with("UPDATE") 
        || sql_upper.starts_with("DELETE");
    
    if is_mutation && !allow_mutations {
        return Err("Mutation queries not allowed in this context".into());
    }
    
    // Execute with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        execute_query(sql, db_pool)
    ).await??;
    
    Ok(result)
}

async fn execute_query(
    sql: &str,
    db_pool: &sqlx::PgPool,
) -> Result<String, sqlx::Error> {
    let rows = sqlx::query(sql)
        .fetch_all(db_pool)
        .await?;
    
    // Convert rows to JSON or formatted string
    Ok(format!("{:?}", rows))
}
```

## 5. Prompt Templating and Management

Build effective prompts for your agent:

```rust
pub struct PromptBuilder {
    system_prompt: String,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            system_prompt: r#"
You are a PostgreSQL database assistant. You help users:
- Query their database with SELECT statements
- Export data in various formats (CSV, JSON)
- Generate database migrations
- Execute DML operations (INSERT, UPDATE, DELETE)

You have access to these tools:
- query_database: Run SELECT queries
- run_dml: Execute INSERT/UPDATE/DELETE
- export_data: Export query results
- generate_migration: Create migration files
- get_schema: View table schemas

Always:
1. Understand the user's intent
2. Check the schema if needed
3. Use appropriate tools
4. Validate queries before execution
5. Explain what you're doing

For safety:
- Ask for confirmation before mutations
- Avoid DROP or TRUNCATE without explicit permission
- Limit large result sets
            "#.to_string(),
        }
    }
    
    pub fn build_prompt(
        &self,
        user_query: &str,
        context: &AgentContext,
    ) -> String {
        let mut prompt = self.system_prompt.clone();
        
        // Add schema context if available
        if let Some(schema) = &context.database_schema {
            prompt.push_str(&format!("\n\nDatabase Schema:\n{}\n", schema));
        }
        
        // Add conversation history
        if !context.messages.is_empty() {
            prompt.push_str("\n\nConversation History:\n");
            prompt.push_str(&context.get_conversation_history());
        }
        
        // Add current query
        prompt.push_str(&format!("\n\nUser Query: {}\n", user_query));
        
        // Add guidance based on recent steps
        if context.steps.len() > 5 {
            prompt.push_str("\nNote: You've taken several steps. Consider providing a final answer.\n");
        }
        
        prompt
    }
}
```

## Key Dependencies (Cargo.toml)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "json"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
```

This architecture gives you a solid foundation for a PostgreSQL AI agent. The key is keeping the agent loop clean, managing context carefully, and implementing robust tool execution with proper error handling and safety checks.