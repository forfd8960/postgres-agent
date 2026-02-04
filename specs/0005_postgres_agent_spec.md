# PostgreSQL AI Agent - Technical Specification

## Document Information

| Attribute | Value |
|-----------|-------|
| Version | 1.0.0 |
| Status | Draft |
| Last Updated | 2026-02-04 |

---

## 1. Executive Summary

This document provides a comprehensive technical specification for building a Rust-based PostgreSQL AI Agent. The agent enables users to interact with PostgreSQL databases using natural language through an interactive Terminal User Interface (TUI). The system translates user intent into safe SQL operations, executes them via PostgreSQL, and returns results with context-aware guidance.

### 1.1 Core Objectives

- **Natural Language Interface**: Enable developers, analysts, and DBAs to query and manage PostgreSQL databases using conversational language.
- **Safety by Design**: Implement layered security including read-only mode, confirmation workflows, SQL preview, and audit logging.
- **Responsive TUI**: Provide an interactive chat-like interface with real-time feedback and pagination.
- **Multi-Database Support**: Allow seamless switching between configured database profiles.
- **Context Awareness**: Maintain session state for multi-step workflows and schema understanding.

---

## 2. Architecture Overview

### 2.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           PostgreSQL AI Agent                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                          Presentation Layer                              │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐ │ │
│  │  │    TUI       │  │    CLI       │  │     Configuration            │ │ │
│  │  │  (ratatui)   │  │   (clap)     │  │      Management             │ │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                      Agent Orchestration Layer                           │ │
│  │  ┌────────────────┐  ┌────────────────┐  ┌──────────────────────────┐ │ │
│  │  │  ReAct Engine  │  │   Context      │  │   Multi-Agent             │ │ │
│  │  │  (Reason→Act)  │  │   Manager      │  │   Coordinator             │ │ │
│  │  └────────────────┘  └────────────────┘  └──────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                       Tool & Capability Layer                             │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │ │
│  │  │   SQL    │ │ Schema   │ │ Migration│ │ Index    │ │ Performance  │  │ │
│  │  │ Executor │ │ Discovery│ │ Manager  │ │ Manager  │ │ Analyzer     │  │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────────┘  │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                    │                                          │
│                                    ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────────────┐ │
│  │                      Infrastructure Layer                               │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────────┐  │ │
│  │  │    LLM       │  │  PostgreSQL  │  │   Observability              │  │ │
│  │  │  Provider    │  │  Connection  │  │   (tracing)                 │  │ │
│  │  └──────────────┘  └──────────────┘  └──────────────────────────────┘  │ │
│  └─────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Design Principles

The architecture follows these fundamental principles:

| Principle | Application |
|-----------|-------------|
| **Modular Specialist** | Decompose functionality into focused modules (SQL generation, schema discovery, execution) |
| **Actor Model** | Each subsystem owns its state, communicates via channels |
| **Type Safety** | Leverage Rust's type system to make illegal states unrepresentable |
| **Fail-Safe Defaults** | Default to read-only mode with explicit opt-in for mutations |
| **Observable** | Structured logging and audit trails for all operations |

---

## 3. Module Specifications

### 3.1 Core Agent Module (`agent-core`)

**Purpose**: Implements the ReAct (Reason + Act) reasoning loop that drives the agent's decision-making process.

**Key Responsibilities**:
- Orchestrate the agent's thought-action-observation cycle
- Manage conversation state and context persistence
- Coordinate tool invocations and result processing

```rust
/// The core agent that implements the ReAct reasoning loop.
#[derive(Debug)]
pub struct PostgresAgent {
    /// LLM client for generating responses and tool calls
    llm_client: LlmClient,
    /// Context manager for conversation state
    context: AgentContext,
    /// Tool registry for available capabilities
    tools: ToolRegistry,
    /// Maximum iterations before forcing termination
    max_iterations: NonZeroU32,
    /// Configuration for safety and behavior
    config: AgentConfig,
}

/// Configuration for agent behavior
#[derive(Debug, Clone, TypedBuilder)]
pub struct AgentConfig {
    /// Maximum number of reasoning iterations
    #[builder(default = NonZeroU32::new(10))]
    pub max_iterations: NonZeroU32,
    /// Whether to require confirmation for mutations
    #[builder(default = true)]
    pub require_confirmation: bool,
    /// Whether to show SQL preview before execution
    #[builder(default = true)]
    pub show_sql_preview: bool,
    /// Safety level for operations
    #[builder(default = SafetyLevel::Balanced)]
    pub safety_level: SafetyLevel,
}

/// Safety levels controlling agent behavior
#[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SafetyLevel {
    /// Maximum safety - read-only, no modifications
    #[default]
    ReadOnly,
    /// Balanced safety - confirmations for DML/DDL
    Balanced,
    /// Permissive - faster execution with minimal checks
    Permissive,
}
```

**ReAct Loop Implementation**:

```rust
impl PostgresAgent {
    /// Execute the agent's reasoning loop for a user query.
    ///
    /// # Errors
    /// Returns `AgentError::MaxIterationsExceeded` if the loop exceeds configured iterations.
    /// Returns `AgentError::LlmUnavailable` if the LLM service is unreachable.
    #[tracing::instrument(skip_all, fields(query = %query))]
    pub async fn run(
        &mut self,
        query: &str,
    ) -> Result<AgentResponse, AgentError> {
        let mut iterations = 0u32;
        let span = tracing::info_span!("agent_execution", query);

        self.context.add_user_message(query);

        loop {
            iterations += 1;

            // Check iteration limit
            if iterations > self.config.max_iterations.get() {
                return Err(AgentError::MaxIterationsExceeded {
                    query: query.to_string(),
                    iterations,
                });
            }

            // Step 1: Generate reasoning and determine next action
            let decision = self
                .llm_client
                .generate_decision(&self.context)
                .await
                .context("LLM decision generation failed")?;

            // Step 2: If final answer, return it
            if let AgentDecision::FinalAnswer(answer) = decision {
                return Ok(AgentResponse::Answer(answer));
            }

            // Step 3: Execute tool call
            if let AgentDecision::ToolCall(tool_call) = decision {
                let observation = self.execute_tool(&tool_call).await?;

                // Step 4: Add observation to context for next iteration
                self.context.add_tool_execution(&tool_call, &observation);
            }
        }
    }
}

/// Decision made by the agent after reasoning
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum AgentDecision {
    /// Continue reasoning, no tool call needed
    #[serde(rename = "reasoning")]
    Reasoning {
        /// The reasoning trace
        thought: String,
    },
    /// Execute a tool call
    #[serde(rename = "tool_call")]
    ToolCall(ToolCall),
    /// Provide final answer to user
    #[serde(rename = "final_answer")]
    FinalAnswer(String),
}
```

### 3.2 LLM Integration Module (`llm-provider`)

**Purpose**: Abstract LLM interactions behind a unified interface, using `async-openai` for OpenAI-compatible API access. Supports multiple providers (OpenAI, Anthropic, local models via OpenAI-compatible endpoints).

**Key Responsibilities**:
- Provider-agnostic LLM client abstraction
- Tool call format handling (OpenAI function calling)
- Response parsing and validation
- Token management and context window handling

**Dependencies**:
```toml
async-openai = "0.2"
```

```rust
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionTool,
        ChatCompletionToolMessage, Role, ToolChoice,
        ToolType, FunctionObject,
        CreateChatCompletionRequestBuilder,
    },
};
use secrecy::{ExposeSecret, Secret};

/// Unified LLM client interface supporting multiple providers.
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    /// Generate a decision based on the current context.
    async fn generate_decision(
        &self,
        context: &AgentContext,
    ) -> Result<AgentDecision, LlmError>;

    /// Generate a structured response (e.g., SQL query).
    async fn generate_structured<T: serde::de::DeserializeOwned + Debug>(
        &self,
        prompt: &str,
        schema: &T,
    ) -> Result<T, LlmError>;

    /// Get the provider name and model identifier.
    fn provider_info(&self) -> ProviderInfo;
}

/// OpenAI provider implementation using async-openai.
#[derive(Debug, Clone)]
pub struct OpenAiProvider {
    /// Async OpenAI client
    client: Client<OpenAIConfig>,
    /// Model identifier
    model: String,
    /// Maximum tokens for response
    max_tokens: u32,
    /// Temperature for sampling
    temperature: f32,
}

impl OpenAiProvider {
    /// Create a new OpenAI provider from configuration.
    pub fn new(
        api_key: Secret<String>,
        model: impl Into<String>,
        base_url: Option<impl Into<String>>,
    ) -> Result<Self, ConfigError> {
        let mut config = OpenAIConfig::new().with_api_key(api_key.expose_secret().clone());

        if let Some(url) = base_url {
            config = config.with_base_url(url);
        }

        let client = Client::with_config(config);

        Ok(Self {
            client,
            model: model.into(),
            max_tokens: 4096,
            temperature: 0.0,
        })
    }

    /// Create a request builder for chat completions.
    fn build_request(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTool>,
    ) -> CreateChatCompletionRequest {
        let mut builder = CreateChatCompletionRequestBuilder::default()
            .model(&self.model)
            .messages(messages)
            .max_tokens(self.max_tokens)
            .temperature(self.temperature);

        if !tools.is_empty() {
            builder = builder.tools(tools).tool_choice(ToolChoice::Auto);
        }

        // Note: build() may fail if required fields are missing
        builder.build().expect("Failed to build chat completion request")
    }
}

#[async_trait::async_trait]
impl LlmClient for OpenAiProvider {
    async fn generate_decision(
        &self,
        context: &AgentContext,
    ) -> Result<AgentDecision, LlmError> {
        let messages = context.to_openai_messages()?;
        let tools = self.build_tool_definitions()?;

        let request = self.build_request(messages, tools);

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| LlmError::ApiError {
                message: e.to_string(),
            })?;

        self.parse_decision(response)
    }

    fn provider_info(&self) -> ProviderInfo {
        ProviderInfo {
            provider: "openai".to_string(),
            model: self.model.clone(),
        }
    }
}

/// Context extension for converting to OpenAI message format.
trait ToOpenAiMessages {
    fn to_openai_messages(&self) -> Result<Vec<ChatCompletionRequestMessage>, LlmError>;
}

impl ToOpenAiMessages for AgentContext {
    fn to_openai_messages(&self) -> Result<Vec<ChatCompletionRequestMessage>, LlmError> {
        let mut messages = Vec::new();

        // Add system prompt first
        messages.push(ChatCompletionRequestMessage::System(
            self.system_prompt.clone().into(),
        ));

        // Convert conversation messages
        for msg in &self.messages {
            let role = match msg.role {
                MessageRole::User => Role::User,
                MessageRole::Assistant => Role::Assistant,
                MessageRole::Tool => Role::Tool,
                MessageRole::System => Role::System,
            };

            messages.push(ChatCompletionRequestMessage::User(
                msg.content.clone().into(),
            ));
        }

        // Add tool results as tool messages
        for tool_result in &self.tool_results {
            messages.push(ChatCompletionRequestMessage::Tool(
                ChatCompletionToolMessage {
                    role: Role::Tool,
                    tool_call_id: tool_result.call_id.clone(),
                    content: match &tool_result.result {
                        Ok(result) => serde_json::to_string(result)
                            .unwrap_or_else(|_| "null".to_string()),
                        Err(e) => format!("Error: {}", e),
                    },
                }
                .into(),
            ));
        }

        Ok(messages)
    }
}

impl OpenAiProvider {
    /// Convert tool definitions to OpenAI format.
    fn build_tool_definitions(&self) -> Result<Vec<ChatCompletionTool>, LlmError> {
        let tools = self.tools.get_definitions();
        let mut result = Vec::new();

        for tool in tools {
            let parameters = serde_json::from_value::<serde_json::Value>(tool.parameters)
                .map_err(|e| LlmError::ParseError {
                    field: "parameters".to_string(),
                    error: e.to_string(),
                })?;

            result.push(ChatCompletionTool {
                tool_type: ToolType::Function,
                function: FunctionObject {
                    name: tool.name,
                    description: tool.description,
                    parameters: Some(parameters),
                    strict: None,
                },
            });
        }

        Ok(result)
    }

    /// Parse the OpenAI response into an AgentDecision.
    fn parse_decision(
        &self,
        response: CreateChatCompletionResponse,
    ) -> Result<AgentDecision, LlmError> {
        let message = response
            .choices
            .first()
            .ok_or(LlmError::NoResponse)?
            .message
            .clone();

        // Check for tool calls
        if let Some(tool_calls) = message.tool_calls {
            let tool_call = tool_calls
                .into_iter()
                .next()
                .ok_or_else(|| LlmError::NoToolCall)?;

            return Ok(AgentDecision::ToolCall(ToolCall {
                name: tool_call.function.name,
                arguments: serde_json::from_str(&tool_call.function.arguments)
                    .map_err(|e| LlmError::ParseError {
                        field: "arguments".to_string(),
                        error: e.to_string(),
                    })?,
                call_id: tool_call.id,
            }));
        }

        // Return final answer (content)
        let content = message.content.unwrap_or_default();
        Ok(AgentDecision::FinalAnswer(content))
    }
}

/// Tool definition for LLM providers (internal representation).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    /// Unique identifier for the tool
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON Schema for parameters
    #[serde(default)]
    pub parameters: serde_json::Value,
}

/// Provider information returned by LLM clients.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub provider: String,
    pub model: String,
}
```

### 3.3 Database Connection Module (`db-connection`)

**Purpose**: Manage PostgreSQL connections using sqlx with connection pooling and transaction support.

**Key Responsibilities**:
- Async connection pool management
- Query execution and result mapping
- Transaction support with savepoints
- Schema introspection

```rust
/// PostgreSQL connection pool and operations.
#[derive(Debug, Clone)]
pub struct DbConnection {
    pool: sqlx::PgPool,
    config: DbConnectionConfig,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct DbConnectionConfig {
    /// PostgreSQL connection URL
    pub url: Secret<String>,
    /// Maximum pool size
    #[builder(default = NonZeroU32::new(10))]
    pub max_connections: NonZeroU32,
    /// Connection timeout in seconds
    #[builder(default = 30)]
    pub connect_timeout: u64,
    /// Query execution timeout in seconds
    #[builder(default = 60)]
    pub query_timeout: u64,
}

impl DbConnection {
    /// Execute a SELECT query and return results as JSON.
    ///
    /// # Errors
    /// Returns `DbError::QueryFailed` if the query fails.
    /// Returns `DbError::Timeout` if execution exceeds timeout.
    #[tracing::instrument(skip_all, fields(sql = %sql))]
    pub async fn execute_query(
        &self,
        sql: &str,
    ) -> Result<QueryResult, DbError> {
        // Validate query type
        let normalized = sql.trim_start().to_uppercase();
        if !normalized.starts_with("SELECT") {
            return Err(DbError::NonSelectQuery { sql: sql.to_string() });
        }

        // Execute with timeout
        tokio::time::timeout(
            Duration::from_secs(self.config.query_timeout),
            self.execute_select(sql),
        )
        .await
        .map_err(|_| DbError::Timeout)?
    }

    async fn execute_select(&self, sql: &str) -> Result<QueryResult, DbError> {
        let rows = sqlx::query(sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::QueryFailed {
                sql: sql.to_string(),
                source: Box::new(e),
            })?;

        let columns = rows
            .first()
            .map(|r| r.columns().iter().map(|c| c.name().to_string()).collect())
            .unwrap_or_default();

        let values: Vec<Vec<serde_json::Value>> = rows
            .iter()
            .map(|r| {
                r.columns()
                    .iter()
                    .map(|c| map_value(r, c))
                    .collect()
            })
            .collect();

        Ok(QueryResult {
            columns,
            rows: values,
            row_count: rows.len(),
        })
    }

    /// Introspect database schema.
    ///
    /// # Errors
    /// Returns `DbError::ConnectionLost` if the pool is disconnected.
    #[tracing::instrument(skip_all)]
    pub async fn get_schema(
        &self,
        table_filter: Option<&str>,
    ) -> Result<DatabaseSchema, DbError> {
        let tables = match table_filter {
            Some(filter) => sqlx::query_as!(
                SchemaTable,
                r#"
                SELECT table_name, table_schema, table_type
                FROM information_schema.tables
                WHERE table_schema = 'public'
                AND ($1 IS NULL OR table_name LIKE $1 || '%')
                ORDER BY table_name
                "#,
                filter.filter(|_| !filter.is_empty())
            )
            .fetch_all(&self.pool)
            .await?,
            None => sqlx::query_as!(
                SchemaTable,
                r#"
                SELECT table_name, table_schema, table_type
                FROM information_schema.tables
                WHERE table_schema = 'public'
                ORDER BY table_name
                "#
            )
            .fetch_all(&self.pool)
            .await?,
        };

        let mut columns = HashMap::String, Vec<ColumnInfo>>();

        for table in &tables {
            let cols = sqlx::query_as!(
                ColumnInfo,
                r#"
                SELECT
                    column_name,
                    data_type,
                    is_nullable,
                    column_default,
                    character_maximum_length,
                    numeric_precision,
                    numeric_scale
                FROM information_schema.columns
                WHERE table_schema = 'public' AND table_name = $1
                ORDER BY ordinal_position
                "#,
                table.table_name
            )
            .fetch_all(&self.pool)
            .await?;

            columns.insert(table.table_name.clone(), cols);
        }

        // Fetch foreign keys
        let foreign_keys = sqlx::query_as!(
            ForeignKeyInfo,
            r#"
            SELECT
                tc.constraint_name,
                tc.table_name,
                kcu.column_name,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
            WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = 'public'
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(DatabaseSchema {
            tables,
            columns,
            foreign_keys,
        })
    }
}

/// Result of a query execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    /// Column names
    pub columns: Vec<String>,
    /// Row data as JSON values
    pub rows: Vec<Vec<serde_json::Value>>,
    /// Number of rows returned
    pub row_count: usize,
}
```

### 3.4 Tool Registry Module (`tools`)

**Purpose**: Define and execute the capabilities available to the agent.

**Key Responsibilities**:
- Tool definition and registration
- Tool execution with validation
- Permission and safety checks

```rust
/// Registry of available tools for the agent.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Register a new tool.
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = T::NAME.to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    /// Execute a tool by name.
    #[tracing::instrument(skip_all, fields(tool_name = %name))]
    pub async fn execute(
        &self,
        name: &str,
        args: &serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::NotFound {
                tool_name: name.to_string(),
            })?;

        // Check permissions
        tool.check_permissions(ctx)?;

        // Execute with timeout
        let result = tokio::time::timeout(
            ctx.timeout.unwrap_or(DEFAULT_TOOL_TIMEOUT),
            tool.execute(args),
        )
        .await
        .map_err(|_| ToolError::Timeout)?
        .map_err(ToolError::ExecutionFailed)?;

        Ok(result)
    }
}

/// Trait for tool implementations.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool name.
    const NAME: &'static str;

    /// Tool description for LLM.
    fn definition(&self) -> ToolDefinition;

    /// Check if the tool can be executed with current permissions.
    fn check_permissions(&self, ctx: &ToolContext) -> Result<(), ToolError>;

    /// Execute the tool with given arguments.
    async fn execute(&self, args: &serde_json::Value) -> Result<serde_json::Value, ToolError>;
}

/// Context provided during tool execution.
#[derive(Debug, Clone)]
pub struct ToolContext {
    /// Database connection
    pub db: DbConnection,
    /// Safety configuration
    pub safety: SafetyContext,
    /// Execution timeout override
    pub timeout: Option<Duration>,
}

/// Safety context for tool execution.
#[derive(Debug, Clone)]
pub struct SafetyContext {
    /// Current safety level
    pub level: SafetyLevel,
    /// Whether the current session is read-only
    pub read_only: bool,
    /// Required confirmation level
    pub confirmation_required: ConfirmationLevel,
}

/// Confirmation level for destructive operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationLevel {
    /// No confirmation needed
    None,
    /// Simple confirmation (y/n)
    Simple,
    /// Type-specific confirmation (must type "DELETE" to delete)
    Typed,
    /// Requires explicit approval from admin
    AdminApproval,
}
```

### 3.5 TUI Module (`tui`)

**Purpose**: Interactive terminal user interface using ratatui.

**Key Responsibilities**:
- Chat-like conversation interface
- Results pane with pagination
- Command palette and status bar
- Configuration management UI

```rust
/// Main TUI application.
#[derive(Debug)]
pub struct PostgresAgentTui {
    /// Terminal backend
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Application state
    state: AppState,
    /// Event input handling
    events: EventHandler,
    /// Agent instance
    agent: PostgresAgent,
}

impl PostgresAgentTui {
    /// Run the TUI application.
    #[tracing::instrument(skip_all)]
    pub async fn run(&mut self) -> Result<(), TuiError> {
        self.terminal.clear()?;
        loop {
            self.terminal.draw(|f| self.render(f))?;

            match self.events.next().await {
                Event::Key(key) => self.handle_key_event(key).await?,
                Event::Resize(width, height) => {
                    self.state.size = Size { width, height };
                }
                Event::Mouse(_) => {}
                Event::Tick => {}
            }
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Status bar
                Constraint::Min(1),      // Main content
                Constraint::Length(10), // Input area
            ])
            .split(frame.size());

        // Status bar
        self.render_status_bar(frame, chunks[0]);

        // Main content (chat or results)
        self.render_main_content(frame, chunks[1]);

        // Input area
        self.render_input(frame, chunks[2]);
    }
}

/// Application state during TUI session.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    /// Current conversation
    conversation: Conversation,
    /// Active database profile
    active_db: Option<DatabaseProfile>,
    /// Current view mode
    view: ViewMode,
    /// Pagination state
    pagination: PaginationState,
    /// Command palette visibility
    show_command_palette: bool,
    /// Loading states
    loading: LoadingState,
}

/// Available views in the TUI.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ViewMode {
    /// Chat/conversation view
    #[default]
    Chat,
    /// Query results view
    Results,
    /// Schema browser
    Schema,
    /// Configuration
    Settings,
    /// Command palette
    CommandPalette,
}
```

### 3.6 Configuration Module (`config`)

**Purpose**: Manage application configuration from files and environment variables.

**Key Responsibilities**:
- YAML configuration parsing with validation
- Multiple database profile management
- Environment variable overrides
- Configuration hot-reload support

```rust
/// Application configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// LLM provider configuration
    #[serde(default)]
    pub llm: LlmConfig,
    /// Database profiles
    #[serde(default)]
    pub databases: Vec<DatabaseProfile>,
    /// Agent behavior settings
    #[serde(default)]
    pub agent: AgentConfig,
    /// Safety and security settings
    #[serde(default)]
    pub safety: SafetyConfig,
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Database profile configuration.
#[derive(Debug, Clone, TypedBuilder, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseProfile {
    /// Unique profile name
    #[builder(setter(into))]
    pub name: String,
    /// Connection URL (supports secrets management)
    #[builder(setter(into))]
    pub url: Secret<String>,
    /// Optional display name
    #[builder(default, setter(into))]
    pub display_name: Option<String>,
    /// SSL mode preference
    #[builder(default = SslMode::Prefer)]
    pub ssl_mode: SslMode,
    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    #[builder(default = 30)]
    pub connect_timeout: u64,
}

/// LLM provider configuration.
#[derive(Debug, Clone, TypedBuilder, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    /// Provider type (openai, anthropic, ollama, etc.)
    #[builder(default = "openai")]
    pub provider: String,
    /// API base URL for custom endpoints
    #[builder(default, setter(into))]
    pub base_url: Option<Url>,
    /// API key (supports env:// prefix for env var lookup)
    #[builder(default, setter(into))]
    pub api_key: Option<Secret<String>>,
    /// Model identifier
    #[builder(default = "gpt-4o")]
    pub model: String,
    /// Temperature for sampling (0.0 to 2.0)
    #[serde(default = "default_temperature")]
    #[builder(default = 0.0)]
    pub temperature: f32,
    /// Maximum tokens in response
    #[serde(default = "default_max_tokens")]
    #[builder(default = 4096)]
    pub max_tokens: u32,
}
```

### 3.7 Safety & Audit Module (`safety`)

**Purpose**: Implement safety checks, confirmation workflows, and audit logging.

**Key Responsibilities**:
- SQL validation and safety analysis
- Confirmation workflow management
- Audit trail creation
- PII detection and redaction

```rust
/// Safety validator for SQL operations.
#[derive(Debug)]
pub struct SafetyValidator {
    /// Blacklisted SQL patterns
    blacklist: Vec<Regex>,
    /// Allowed operations by safety level
    allowed_ops: HashMap<SafetyLevel, HashSet<OperationType>>,
    /// PII detector
    pii_detector: PiiDetector,
}

impl SafetyValidator {
    /// Validate a SQL query for safety.
    #[tracing::instrument(skip_all)]
    pub fn validate(&self, sql: &str, ctx: &SafetyContext) -> Result<ValidationResult, SafetyError> {
        // Check for blacklisted patterns
        for pattern in &self.blacklist {
            if pattern.is_match(sql) {
                return Err(SafetyError::BlacklistedPattern {
                    pattern: pattern.to_string(),
                });
            }
        }

        // Determine operation type
        let op_type = self.classify_operation(sql)?;

        // Check if operation is allowed
        if !ctx.is_allowed(op_type) {
            return Err(SafetyError::OperationNotAllowed {
                operation: op_type,
                safety_level: ctx.level,
            });
        }

        // Check read-only mode
        if ctx.read_only && op_type.is_mutation() {
            return Err(SafetyError::ReadOnlyViolation {
                operation: op_type,
            });
        }

        Ok(ValidationResult {
            operation_type: op_type,
            is_safe: true,
            warnings: self.check_warnings(sql),
        })
    }
}

/// Audit logger for all operations.
#[derive(Debug)]
pub struct AuditLogger {
    /// Log destination (file, stdout, remote)
    sink: AuditSink,
    /// Log entry formatter
    formatter: AuditFormatter,
}

impl AuditLogger {
    /// Log an audit event.
    #[tracing::instrument(skip_all)]
    pub fn log(&self, event: AuditEvent) {
        let entry = self.formatter.format(&event);
        self.sink.write(&entry);
    }
}

/// Audit event types.
#[derive(Debug, Clone, serde::Serialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuditEvent {
    /// Query execution
    Query {
        timestamp: chrono::DateTime<chrono::Utc>,
        user: String,
        database: String,
        query: String,
        success: bool,
        duration_ms: u64,
    },
    /// Schema modification
    SchemaChange {
        timestamp: chrono::DateTime<chrono::Utc>,
        user: String,
        database: String,
        operation: String,
        sql: String,
        approved: bool,
    },
    /// Safety violation
    SafetyViolation {
        timestamp: chrono::DateTime<chrono::Utc>,
        user: String,
        query: String,
        reason: String,
    },
}
```

---

## 4. Data Flow & Interaction Patterns

### 4.1 Query Execution Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Natural Language Query Flow                          │
└─────────────────────────────────────────────────────────────────────────────┘

   User Input
       │
       ▼
┌─────────────────┐
│  TUI Input      │  Collect user query
│  Handler        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Context        │  Build context with:
│  Builder        │  - Conversation history
│                 │  - Current schema (cached)
│                 │  - Tool definitions
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  ReAct Engine   │  Loop:
│                 │  1. Generate thought
│                 │  2. Decide action
│                 │  3. Execute tool
│                 │  4. Observe result
│                 │  5. Repeat or finish
└────────┬────────┘
         │
         ├── Tool: query_database ──┐
         │                          │
         ▼                          ▼
┌─────────────────┐    ┌─────────────────┐
│  SQL Generator  │    │  Schema         │
│  (LLM)          │    │  Discovery      │
│                 │    │  (sqlx)         │
└────────┬────────┘    └─────────────────┘
         │
         ▼
┌─────────────────┐
│  Safety         │  Validate:
│  Validator      │  - Blacklist patterns
│                 │  - Permission check
│                 │  - Confirmation need
└────────┬────────┘
         │
         ├─ Needs confirmation ──▶ User Approval ─┐
         │                                         │
         ▼                                         ▼
┌─────────────────┐                    ┌─────────────────┐
│  SQL Executor   │                    │  Cancel /        │
│  (sqlx)         │                    │  Modify          │
└────────┬────────┘                    └─────────────────┘
         │
         ▼
┌─────────────────┐
│  Result         │  Format and return:
│  Formatter      │  - Table output
│                 │  - JSON/CSV export
│                 │  - Error message
└────────┬────────┘
         │
         ▼
   TUI Display
```

### 4.2 Context Management Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Context Lifecycle                                   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                           Context Scope Levels                             │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Session Context (Session lifetime)                             │   │
│  │  - Database profile selection                                   │   │
│  │  - Safety configuration                                         │   │
│  │  - Conversation history (windowed)                             │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
│                              ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Turn Context (Single query)                                    │   │
│  │  - Current query                                                │   │
│  │  - Generated SQL (if different)                                │   │
│  │  - Tool execution history                                       │   │
│  │  - Intermediate observations                                    │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                              │                                             │
│                              ▼                                             │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Schema Context (Cached, refreshable)                          │   │
│  │  - Table definitions                                            │   │
│  │  - Column metadata                                              │   │
│  │  - Foreign key relationships                                    │   │
│  │  - Index information                                            │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 4.3 Error Recovery Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Error Recovery & Retry Flow                          │
└─────────────────────────────────────────────────────────────────────────────┘

   Query Execution Error
           │
           ▼
   ┌───────────────┐
   │ Classify      │  Transient?
   │ Error Type    │  (network, timeout)
   └───────┬───────┘
           │
     ┌─────┴─────┐
     │           │
  Transient   Permanent
     │           │
     ▼           ▼
┌────────┐  ┌─────────────┐
│ Retry  │  │  SQL Error  │  (syntax, missing column)
│ N times│  │  Classifier │
└───┬────┘  └──────┬──────┘
    │               │
    │           ┌───┴────────┐
    │           │            │
    │      Syntax    Semantic
    │      Error     Error
    │           │            │
    │           ▼            ▼
    │      ┌─────────┐  ┌──────────────┐
    │      │ Parse   │  │ Self-Heal    │
    │      │ Fix     │  │ (ReAct)      │
    │      └────┬────┘  └──────┬───────┘
    │           │               │
    │           └───────┬───────┘
    │                   ▼
    │           ┌───────────────┐
    │           │ Return to     │
    │           │ ReAct Loop    │
    │           └───────────────┘
    │                   │
    │                   ▼
    │           ┌───────────────┐
    │           │ Fallback:     │
    │           │ Ask user      │
    │           │ clarification │
    │           └───────────────┘
    │                   │
    └───────────────────┘
```

---

## 5. Core Traits & Data Structures

### 5.1 Core Trait Hierarchy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Core Trait Hierarchy                               │
└─────────────────────────────────────────────────────────────────────────────┘

                              Send + Sync
                                   │
                                   ▼
                           ┌───────────────┐
                           │   AgentTrait  │  Base capability interface
                           └───────┬───────┘
                                   │
               ┌───────────────────┼───────────────────┐
               │                   │                   │
               ▼                   ▼                   ▼
       ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
       │  QueryAgent   │   │  SchemaAgent  │   │  MigrateAgent │
       └───────┬───────┘   └───────┬───────┘   └───────┬───────┘
               │                   │                   │
               │                   │                   │
               ▼                   ▼                   ▼
       ┌───────────────┐   ┌───────────────┐   ┌───────────────┐
       │ execute_query │   │ get_schema    │   │ apply_migration│
       │ explain_query │   │ list_tables   │   │ rollback       │
       │ export_data   │   │ analyze_rel   │   │ diff_schema    │
       └───────────────┘   └───────────────┘   └───────────────┘
```

### 5.2 Key Type Definitions

```rust
// === Core Agent Types ===

/// Represents a single message in the conversation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Role (user, assistant, tool, system)
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Timestamp
    #[serde(default = "Utc::now")]
    pub timestamp: DateTime<Utc>,
    /// Optional: SQL that was generated
    #[serde(default)]
    pub generated_sql: Option<String>,
}

/// Role of a message in the conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageRole {
    User,
    Assistant,
    Tool,
    System,
}

/// A tool call made by the agent.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// Tool name
    pub name: String,
    /// Tool arguments (JSON)
    pub arguments: serde_json::Value,
    /// Call ID for tracking
    pub call_id: String,
}

/// Result of a tool execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    /// Original call ID
    pub call_id: String,
    /// Tool name
    pub tool: String,
    /// Result data (JSON)
    pub result: serde_json::Value,
    /// Whether execution was successful
    pub success: bool,
    /// Error message if failed
    #[serde(default)]
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

// === Database Types ===

/// Represents a database schema element.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSchema {
    /// Tables in the schema
    pub tables: Vec<SchemaTable>,
    /// Columns by table name
    pub columns: HashMap<String, Vec<ColumnInfo>>,
    /// Foreign key relationships
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

/// Table information from schema.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaTable {
    pub table_name: String,
    pub table_schema: String,
    pub table_type: TableType,
}

/// Column information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    #[serde(default)]
    pub column_default: Option<String>,
    #[serde(default)]
    pub character_maximum_length: Option<i64>,
    #[serde(default)]
    pub numeric_precision: Option<i64>,
    #[serde(default)]
    pub numeric_scale: Option<i64>,
}

/// Type of table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TableType {
    BaseTable,
    View,
    ForeignTable,
}

/// Foreign key relationship.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignKeyInfo {
    pub constraint_name: String,
    pub table_name: String,
    pub column_name: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
}
```

---

## 6. Async & Concurrency Model

### 6.1 Actor Architecture

The system uses the Actor model for organizing concurrent components. Each major subsystem runs as an actor with its own state, communicating via message passing.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Actor Communication Model                           │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│    TUI Actor    │◄───────►│  Orchestrator   │◄───────►│    LLM Actor    │
│  (ratatui)      │   UI    │  Actor          │   API   │  (async-openai) │
│                 │   msgs  │                 │   calls │                 │
└────────┬────────┘         └────────┬────────┘         └────────┬────────┘
         │                           │                           │
         │    ┌──────────────────────┴──────────────────────┐    │
         │    │                                              │    │
         ▼    ▼                                              ▼    ▼
┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
│   DB Actor      │         │   Tool Actor    │         │ Audit Actor     │
│  (sqlx pool)    │         │  (tool registry)│         │ (audit logging) │
│                 │         │                 │         │                 │
└─────────────────┘         └─────────────────┘         └─────────────────┘
```

### 6.2 Concurrency Patterns

```rust
// === Agent Execution with JoinSet for Parallel Tool Calls ===

impl PostgresAgent {
    /// Execute multiple independent tool calls in parallel.
    async fn execute_parallel_tools(
        &self,
        calls: Vec<ToolCall>,
    ) -> Vec<ToolResult> {
        let mut set = JoinSet::new();

        for call in calls {
            set.spawn(self.execute_tool_async(call));
        }

        let mut results = Vec::with_capacity(calls.len());
        while let Some(result) = set.join_next().await {
            match result {
                Ok(r) => results.push(r),
                Err(e) => {
                    tracing::error!(?e, "Tool execution panicked");
                    // Continue with other executions
                }
            }
        }

        results
    }
}

// === Shutdown Signal Handling with AtomicBool ===

impl PostgresAgent {
    /// Run the agent with graceful shutdown support.
    pub async fn run_with_shutdown(
        &mut self,
        shutdown: Arc<AtomicBool>,
    ) -> Result<(), AgentError> {
        loop {
            tokio::select! {
                _ = shutdown.cchanged() => {
                    tracing::info!("Shutdown signal received");
                    break;
                }
                event = self.events.next() => {
                    if let Some(query) = event {
                        self.handle_query(query).await?;
                    }
                }
            }
        }
        Ok(())
    }
}

// === Rate Limiting for LLM Calls ===

#[derive(Debug)]
pub struct RateLimiter {
    /// Permits remaining
    permits: Arc<Semaphore>,
    /// Rate limit duration
    duration: Duration,
    /// Last refill timestamp
    last_refill: AtomicI64,
}

impl RateLimiter {
    /// Acquire a permit for an LLM call.
    async fn acquire(&self) -> Result<(), RateLimitError> {
        let permit = self
            .permits
            .try_acquire()
            .map_err(|_| RateLimitError::Exceeded)?;

        // Check if refill is needed
        let now = Utc::now().timestamp();
        let last = self.last_refill.load(Ordering::SeqCst);

        if now - last >= self.duration.as_secs() as i64 {
            // Refill permits
            self.permits.add(self.max_permits);
            self.last_refill.store(now, Ordering::SeqCst);
        }

        Ok(())
    }
}
```

### 6.3 Channel-Based Communication

```rust
// === Message Types for Actor Communication ===

/// Messages handled by the orchestrator actor.
#[derive(Debug)]
pub enum OrchestratorMessage {
    /// Process a user query
    Query {
        query: String,
        response_tx: oneshot::Sender<Result<AgentResponse, AgentError>>,
    },
    /// Switch database profile
    SwitchDatabase {
        profile_name: String,
        response_tx: oneshot::Sender<Result<(), ConfigError>>,
    },
    /// Get current state
    GetState {
        response_tx: oneshot::Sender<AgentState>,
    },
}

/// Channel for orchestrator communication.
pub type OrchestratorChannel = mpsc::Sender<OrchestratorMessage>;

impl AgentOrchestrator {
    /// Handle incoming messages.
    async fn handle_message(&mut self, msg: OrchestratorMessage) {
        match msg {
            OrchestratorMessage::Query { query, response_tx } => {
                let result = self.process_query(&query).await;
                let _ = response_tx.send(result);
            }
            OrchestratorMessage::SwitchDatabase { profile_name, response_tx } => {
                let result = self.switch_database(&profile_name).await;
                let _ = response_tx.send(result);
            }
            OrchestratorMessage::GetState { response_tx } => {
                let _ = response_tx.send(self.state.clone());
            }
        }
    }
}
```

---

## 7. Error Handling Strategy

### 7.1 Error Hierarchy

```rust
// === Error Type Hierarchy ===

/// Root error type for the application.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Agent-specific errors
    #[error("agent error: {source}")]
    Agent {
        #[from]
        source: AgentError,
    },
    /// Database errors
    #[error("database error: {source}")]
    Database {
        #[from]
        source: DbError,
    },
    /// LLM provider errors
    #[error("LLM error: {source}")]
    Llm {
        #[from]
        source: LlmError,
    },
    /// Configuration errors
    #[error("configuration error: {source}")]
    Configuration {
        #[from]
        source: ConfigError,
    },
    /// Safety/security violations
    #[error("safety violation: {reason}")]
    Safety {
        reason: String,
    },
    /// TUI errors
    #[error("TUI error: {source}")]
    Tui {
        #[from]
        source: TuiError,
    },
}

/// Agent-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Maximum iterations exceeded
    #[error("max iterations ({iterations}) exceeded for query: {query}")]
    MaxIterationsExceeded {
        query: String,
        iterations: u32,
    },
    /// Invalid tool call
    #[error("invalid tool call: {details}")]
    InvalidToolCall {
        details: String,
    },
    /// Tool execution failed
    #[error("tool execution failed: {tool_name} - {reason}")]
    ToolExecutionFailed {
        tool_name: String,
        reason: String,
    },
    /// Context too large for model
    #[error("context exceeds model limit: {size} tokens")]
    ContextTooLarge {
        size: usize,
        limit: usize,
    },
}

/// Database-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Connection failed
    #[error("failed to connect to database: {source}")]
    ConnectionFailed {
        #[source]
        source: sqlx::Error,
    },
    /// Query execution failed
    #[error("query failed: {sql} - {source}")]
    QueryFailed {
        sql: String,
        #[source]
        source: sqlx::Error,
    },
    /// Non-SELECT query attempted in read-only mode
    #[error("non-SELECT query not allowed in read-only mode")]
    NonSelectQuery {
        sql: String,
    },
    /// Query timeout
    #[error("query exceeded timeout of {timeout}s")]
    Timeout {
        timeout: u64,
    },
}

/// LLM provider errors.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    /// API error from OpenAI
    #[error("OpenAI API error: {message}")]
    ApiError {
        message: String,
    },
    /// Response parsing failed
    #[error("failed to parse LLM response: {field} - {error}")]
    ParseError {
        field: String,
        error: String,
    },
    /// No response from LLM
    #[error("no response received from LLM")]
    NoResponse,
    /// No tool call in response
    #[error("expected tool call but none provided")]
    NoToolCall,
    /// Rate limited
    #[error("rate limited: retry after {retry_after}s")]
    RateLimited {
        retry_after: u64,
    },
    /// Context too large
    #[error("context exceeds model limit: {size} tokens")]
    ContextTooLarge {
        size: usize,
        limit: usize,
    },
}
```

### 7.2 Error Recovery Strategies

| Error Category | Recovery Strategy | Implementation |
|----------------|------------------|----------------|
| Network timeout | Exponential backoff retry | `retry` crate with jitter |
| LLM rate limit | Wait and retry | Token bucket with sleep |
| SQL syntax error | Self-correction via ReAct | Pass error to LLM for fix |
| Connection lost | Reconnect with exponential backoff | Circuit breaker pattern |
| Context overflow | Summarize history | Truncate oldest messages |
| Invalid tool call | Log and skip | Error aggregation |

```rust
// === Retry Logic with Exponential Backoff ===

#[tracing::instrument(skip_all)]
pub async fn with_retry<T, F, Fut>(
    operation: F,
    config: RetryConfig,
) -> Result<T, OperationError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, OperationError>>,
{
    let mut attempt = 0u32;
    let mut delay = config.initial_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if config.retryable(&e) && attempt < config.max_attempts => {
                attempt += 1;
                tracing::warn!(
                    attempt,
                    delay_secs = delay.as_secs(),
                    error = %e,
                    "Operation failed, retrying"
                );

                tokio::time::sleep(delay).await;

                // Exponential backoff with jitter
                delay = (delay * config.backoff_multiplier)
                    .min(config.max_delay)
                    .mul_f64(1.0 + rand::thread_rng().gen_range(0.0..0.1));
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## 8. Testing Strategy

### 8.1 Testing Pyramid

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Testing Pyramid                                 │
└─────────────────────────────────────────────────────────────────────────────┘

                                ▲
                               /│\
                              / │ \
                             /  │  \
                            /   │   \
                           /    │    \
                          /     │     \
                         /      │      \
                        /       │       \
                       /        │        \
                      /         │         \
                     /          │          \
                    /           │           \
         ┌─────────┐    ┌─────────┐    ┌─────────┐
         │  Unit   │    │  Unit   │    │  Unit   │
         │  Tests  │    │  Tests  │    │  Tests  │
         │  (70%)  │    │  (70%)  │    │  (70%)  │
         └─────────┘    └─────────┘    └─────────┘
                      Integration Tests (20%)
                      ┌─────────────────┐
                      │  Module         │
                      │  Integration   │
                      └─────────────────┘
                                  │
                                  ▼
                      ┌─────────────────┐
                      │  E2E Tests      │
                      │  (10%)         │
                      │  CLI + TUI      │
                      └─────────────────┘
```

### 8.2 Test Categories

| Level | Focus | Tools | Examples |
|-------|-------|-------|----------|
| Unit | Single functions/components | `#[test]`, `rstest` | SQL validation, prompt building |
| Integration | Module interactions | `#[cfg(test)] mod tests` | Agent + LLM mock, DB + connection |
| E2E | Full user workflows | `std::process::Command` | CLI commands, TUI interactions |

### 8.3 Unit Test Examples

```rust
// === SQL Safety Validator Tests ===

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest]
    #[test]
    #[case("SELECT * FROM users", true)]
    #[case("INSERT INTO users VALUES (1, 'test')", true)]
    #[case("DROP TABLE users", false)]
    #[case("TRUNCATE users", false)]
    fn test_sql_safety_validation(
        #[case] sql: &str,
        #[case] expected_safe: bool,
    ) {
        let validator = SafetyValidator::default();
        let ctx = SafetyContext::balanced();

        let result = validator.validate(sql, &ctx);

        assert_eq!(result.is_ok(), expected_safe);
    }

    #[rstest]
    #[test]
    fn test_read_only_mode_blocks_mutations() {
        let validator = SafetyValidator::default();
        let mut ctx = SafetyContext::balanced();
        ctx.read_only = true;

        let result = validator.validate("UPDATE users SET name = 'test'", &ctx);

        assert!(matches!(result, Err(SafetyError::ReadOnlyViolation { .. })));
    }

    #[rstest]
    #[test]
    fn test_pii_detection_in_queries() {
        let detector = PiiDetector::default();

        assert!(detector.contains_pii("SELECT * FROM users WHERE ssn = '123-45-6789'"));
        assert!(!detector.contains_pii("SELECT * FROM users WHERE id = 1"));
    }
}

// === Context Builder Tests ===

#[cfg(test)]
mod context_tests {
    use super::*;

    #[test]
    fn test_context_window_enforcement() {
        let mut ctx = AgentContext::new(5); // Max 5 messages

        // Add 7 messages (exceeds limit)
        for i in 0..7 {
            ctx.add_user_message(&format!("Message {}", i));
        }

        // Should have oldest 2 dropped
        assert_eq!(ctx.messages.len(), 5);
        assert_eq!(ctx.messages[0].content, "Message 2");
    }

    #[test]
    fn test_conversation_history_format() {
        let mut ctx = AgentContext::default();
        ctx.add_user_message("Hello");
        ctx.add_assistant_message("Hi there!");

        let history = ctx.get_conversation_history();

        assert!(history.contains("user: Hello"));
        assert!(history.contains("assistant: Hi there!"));
    }
}
```

### 8.4 Mock Infrastructure

```rust
// === Mock LLM Provider for Testing ===

#[derive(Debug, Clone)]
pub struct MockLlmClient {
    /// Pre-configured responses
    responses: VecDeque<AgentDecision>,
    /// Call recording
    calls: Arc<Mutex<Vec<AgentContext>>>,
}

#[async_trait::async_trait]
impl LlmClient for MockLlmClient {
    async fn generate_decision(
        &self,
        context: &AgentContext,
    ) -> Result<AgentDecision, LlmError> {
        self.calls.lock().unwrap().push(context.clone());

        self.responses
            .pop_front()
            .ok_or(LlmError::NoResponseConfigured)
    }
}

// === Mock Database for Testing ===

#[derive(Debug, Default)]
pub struct MockDbConnection {
    /// Pre-configured query results
    results: HashMap<String, QueryResult>,
    /// Schema information
    schema: DatabaseSchema,
    /// Call recording
    queries: Arc<Mutex<Vec<String>>>,
}

impl MockDbConnection {
    pub fn with_query(sql: impl Into<String>, result: QueryResult) -> Self {
        let mut this = Self::default();
        this.results.insert(sql.into(), result);
        this
    }
}
```

### 8.5 Fuzz Testing

```rust
// === Property-Based Testing with Proptest ===

proptest! {
    #[test]
    fn test_sql_validation_properties(sql in "SELECT .* FROM \\w+") {
        let validator = SafetyValidator::default();
        let ctx = SafetyContext::balanced();

        let result = validator.validate(&sql, &ctx);

        // SELECT queries should always be allowed in balanced mode
        if sql.to_uppercase().starts_with("SELECT") {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_context_pagination(page in 1..=100usize, page_size in 1..=50usize) {
        let mut ctx = AgentContext::new(page_size);
        for i in 0..200 {
            ctx.add_user_message(&format!("Message {}", i));
        }

        let paginated = ctx.get_paginated(page, page_size);

        assert_eq!(paginated.len(), page_size.min(200 - (page - 1) * page_size));
    }
}
```

---

## 9. Logging & Observability

### 9.1 Structured Logging with Tracing

```rust
// === Tracing Setup ===

#[tracing::instrument(skip_all)]
pub async fn execute_query(
    &self,
    sql: &str,
) -> Result<QueryResult, DbError> {
    tracing::info!(sql = %sql, "Executing query");
    tracing::debug!(?sql, "Detailed query trace");

    let start = Instant::now();
    let result = self.execute_select(sql).await;

    let duration = start.elapsed();
    tracing::info!(
        rows = result.as_ref().map(|r| r.row_count).unwrap_or(0),
        duration_ms = duration.as_millis(),
        "Query executed"
    );

    result
}

// === Log Output Format ===

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: serde_json::Value,
    pub span_context: SpanContext,
}
```

### 9.2 Metrics

```rust
// === Application Metrics ===

#[derive(Debug, Clone)]
pub struct Metrics {
    /// Counter for queries executed
    query_counter: Counter<u64>,
    /// Histogram for query duration
    query_duration: Histogram<f64>,
    /// Counter for errors by type
    error_counter: Counter<u64>,
    /// Gauge for active connections
    active_connections: Gauge<i64>,
}

/// Metrics implementation using prometheus-client
impl Metrics {
    pub fn record_query(&self, success: bool, duration_ms: f64) {
        self.query_counter.inc();
        self.query_duration.observe(duration_ms);

        if !success {
            self.error_counter.with_label(&["query"]).inc();
        }
    }
}
```

### 9.3 Audit Logging

```rust
// === Audit Event Types ===

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum AuditEvent {
    QueryExecuted {
        timestamp: DateTime<Utc>,
        user: String,
        database: String,
        natural_language: String,
        generated_sql: String,
        success: bool,
        duration_ms: u64,
        row_count: usize,
    },

    SchemaModification {
        timestamp: DateTime<Utc>,
        user: String,
        database: String,
        operation: String,
        sql: String,
        confirmation_level: ConfirmationLevel,
        approved: bool,
    },

    SafetyViolation {
        timestamp: DateTime<Utc>,
        query: String,
        reason: String,
        safety_level: SafetyLevel,
    },

    Export {
        timestamp: DateTime<Utc>,
        user: String,
        database: String,
        query: String,
        format: ExportFormat,
        path: PathBuf,
    },
}
```

---

## 10. Dependencies & Crate Selection

### 10.1 Core Dependencies

| Crate | Version | Purpose | Justification |
|-------|---------|---------|---------------|
| `tokio` | 1.x | Async runtime | Industry standard, excellent ergonomics |
| `sqlx` | 0.7.x | PostgreSQL access | Compile-time checks, async-first |
| `ratatui` | 0.26.x | Terminal UI | Ergonomic, feature-rich |
| `clap` | 4.x | CLI argument parsing | Derive-based, flexible |
| `async-openai` | 0.2.x | OpenAI API client | Simple, well-maintained |
| `serde` / `serde_json` | 1.x | Serialization | Standard ecosystem |
| `thiserror` | 2.x | Error types | Derive-based error definitions |
| `anyhow` | 1.x | Application errors | Context-rich error handling |
| `tracing` / `tracing-subscriber` | 0.3.x | Structured logging | First-class async support |
| `tokio::sync` | 1.x | Channels and primitives | Part of tokio |
| `DashMap` | 6.x | Concurrent HashMap | Lock-free reads |

### 10.2 Optional Dependencies

| Crate | Feature | Purpose |
|-------|---------|---------|
| `secrecy` | security | Secret management |
| `refinery` | migrations | Database migration management |
| `regex` | validation | SQL pattern matching |
| `csv` | export | CSV export functionality |
| `chrono` | time | Timestamp handling |

### 10.3 Cargo.toml Configuration

```toml
[package]
name = "postgres-agent"
version = "0.1.0"
edition = "2024"
rust-version = "1.93.0"

[dependencies]
# Core runtime
tokio = { version = "1.49", features = ["rt-multi-thread", "macros", "sync", "tracing"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "postgres", "json"] }

# CLI/TUI
clap = { version = "4", features = ["derive"] }
ratatui = { version = "0.30.0", features = ["crossterm", "serde"] }
crossterm = "0.29"
tui-textarea = "0.4"

# LLM Integration
async-openai = "0.32.4"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# Error Handling
thiserror = "2"
anyhow = "1.0.100"

# Logging & Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-appender = "0.2"

# Concurrency
tokio-stream = "0.1"
futures = "0.3"
dashmap = "6"

# Configuration
config = "0.14"
dotenvy = "0.15"

# Utilities
derive_more = "1"
 secrecy = "0.8"
async-trait = "0.1"

# Testing
rstest = "0.23"
proptest = "1.5"
mockall = "0.13"
wiremock = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "4"

[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"

[features]
# Optional features
vendored = ["tokio/rt-multi-thread"]
opentelemetry = ["tracing-opentelemetry"]
```

---

## 11. Security Considerations

### 11.1 Threat Model

| Threat | Mitigation |
|--------|------------|
| SQL Injection | Parameterized queries, LLM-generated SQL validation |
| Prompt Injection | Input sanitization, context isolation |
| Data Exfiltration | PII detection, output filtering |
| Unauthorized Access | Role-based permissions, connection credentials |
| Credential Exposure | Secrets management, environment variables |

### 11.2 Security Implementation

```rust
// === Input Sanitization ===

pub struct InputSanitizer {
    /// Patterns to block
    blocked_patterns: Vec<Regex>,
    /// Maximum input length
    max_length: usize,
}

impl InputSanitizer {
    pub fn sanitize(&self, input: &str) -> Result<String, SanitizationError> {
        // Length check
        if input.len() > self.max_length {
            return Err(SanitizationError::TooLong {
                length: input.len(),
                limit: self.max_length,
            });
        }

        // Pattern blocking
        for pattern in &self.blocked_patterns {
            if pattern.is_match(input) {
                return Err(SanitizationError::BlockedPattern {
                    pattern: pattern.to_string(),
                });
            }
        }

        Ok(input.to_string())
    }
}

// === PII Detection ===

#[derive(Debug, Default)]
pub struct PiiDetector {
    patterns: Vec<(Regex, PiiType)>,
}

impl PiiDetector {
    /// Check if content contains PII.
    pub fn contains_pii(&self, content: &str) -> bool {
        self.patterns.iter().any(|(re, _)| re.is_match(content))
    }

    /// Redact PII from content.
    pub fn redact(&self, content: &str) -> String {
        let mut result = content.to_string();
        for (re, pii_type) in &self.patterns {
            let replacement = format!("[{}]", pii_type.label());
            result = re.replace_all(&result, replacement).to_string();
        }
        result
    }
}
```

---

## 12. Performance Considerations

### 12.1 Optimization Strategies

| Component | Strategy | Implementation |
|-----------|----------|---------------|
| LLM calls | Request caching | Cache similar queries |
| Schema fetching | Caching | In-memory cache with TTL |
| Database connections | Pooling | sqlx connection pool |
| UI rendering | Batching | Batch updates, reduce redraws |
| Memory usage | Streaming | Paginate large result sets |

### 12.2 Benchmark Setup

```rust
// === Benchmark using Criterion ===

fn bench_query_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_generation");

    group.bench_function("simple_select", |b| {
        b.iter(|| {
            let agent = TestAgent::new();
            let runtime = block_on(agent.generate_query("List all users"));
            assert!(runtime.is_ok());
        })
    });

    group.bench_function("complex_join", |b| {
        b.iter(|| {
            let agent = TestAgent::new();
            let runtime = block_on(agent.generate_query(
                "Get total orders per customer with their latest order date"
            ));
            assert!(runtime.is_ok());
        })
    });

    group.finish();
}
```

---

## 13. Future Extensions

### 13.1 Planned Features (v2+)

- **Multi-Agent Orchestration**: Specialized agents for query, migration, and optimization
- **Local Model Support**: Ollama, LM Studio integration
- **Web UI**: HTTP API with web interface
- **Plugins**: Extensible tool system
- **Team Collaboration**: Shared contexts and query library

### 13.2 Extension Points

```rust
/// Trait for custom tool implementations.
#[async_trait::async_trait]
pub trait ToolExtension: Send + Sync {
    /// Tool metadata.
    fn metadata(&self) -> ExtensionMetadata;

    /// Initialize with agent context.
    async fn initialize(&mut self, context: &AgentContext) -> Result<(), Error>;

    /// Execute custom action.
    async fn execute(
        &self,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, Error>;
}
```

---

## 14. Implementation Roadmap

### Phase 1: MVP (Milestone 1)
- [ ] Core agent with ReAct loop
- [ ] Basic SQL generation and execution
- [ ] Simple TUI with chat interface
- [ ] Single database configuration
- [ ] Read-only mode with safety checks

### Phase 2: Core Features (Milestone 2)
- [ ] Multi-database profile support
- [ ] Confirmation workflows for DML/DDL
- [ ] Export to CSV/JSON
- [ ] Audit logging
- [ ] Configuration management

### Phase 3: Advanced Features (Milestone 3)
- [ ] Migration generation and execution
- [ ] Index creation recommendations
- [ ] Performance analysis (EXPLAIN)
- [ ] Schema browser UI
- [ ] Advanced error recovery

---

## 15. References

1. ReAct: Synergizing Reasoning and Acting in Language Models - [arXiv](https://arxiv.org/abs/2210.03629)
2. LangChain vs LlamaIndex: Comparison for Agentic AI - [ZenML](https://www.zenml.io/blog/llamaindex-vs-langchain)
3. Building AI Agents in Rust - [Refresh Agent](https://refreshagent.com/engineering/building-ai-agents-in-rust)
4. PostgreSQL Best Practices for AI Agents - [Supabase](https://supabase.com/blog/postgres-best-practices-for-ai-agents)
5. sqlx Documentation - [docs.rs](https://docs.rs/sqlx)
6. ratatui Documentation - [ratatui.rs](https://ratatui.rs/)
