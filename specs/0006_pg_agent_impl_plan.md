# PostgreSQL AI Agent - Implementation Plan

## Document Information

| Attribute | Value |
|-----------|-------|
| Version | 1.0.0 |
| Status | Draft |
| Last Updated | 2026-02-04 |
| Based On | 0005_postgres_agent_spec.md |

---

## 1. Project Overview

This implementation plan translates the technical specification into actionable development steps. The PostgreSQL AI Agent will be built as a Rust-based CLI/TUI application that enables users to interact with PostgreSQL databases using natural language.

### 1.1 High-Level Goals

- **Phase 1 (MVP)**: Core agent with ReAct loop, basic SQL generation, simple TUI, read-only safety
- **Phase 2 (Core Features)**: Multi-database support, confirmation workflows, export functionality, audit logging
- **Phase 3 (Advanced)**: Migration management, index recommendations, performance analysis, schema browser

### 1.2 Technology Stack

| Layer | Technology | Justification |
|-------|------------|---------------|
| Runtime | Tokio 1.x | Industry-standard async runtime |
| Database | sqlx 0.8+ | Compile-time checks, PostgreSQL native |
| CLI | Clap 4.x | Derive-based, flexible |
| TUI | ratatui 0.30+ | Ergonomic, feature-rich |
| LLM | async-openai 0.32+ | OpenAI-compatible API client |
| Error Handling | thiserror + anyhow | Derive-based, context-rich |
| Logging | tracing 0.3  + tracing-subscriber 0.3.22 | Structured, async-first |

---

## 2. Project Structure

```
postgres-agent/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── .env.example
├── README.md
├── CONTRIBUTING.md
├── LICENSE
├──
├── src/
│   ├── main.rs
│   ├── lib.rs
│   │
│   ├── bin/
│   │   └── pg_agent/
│   │       └── main.rs
│   │
│   ├── core/                  # Core agent logic
│   │   ├── mod.rs
│   │   ├── agent.rs           # PostgresAgent, AgentConfig
│   │   ├── context.rs         # AgentContext, Message, Conversation
│   │   ├── decision.rs        # AgentDecision, ToolCall
│   │   └── error.rs           # AgentError enum
│   │
│   ├── llm/                   # LLM provider abstraction
│   │   ├── mod.rs
│   │   ├── client.rs          # LlmClient trait
│   │   ├── openai.rs          # OpenAiProvider implementation
│   │   ├── provider.rs        # ProviderInfo, ProviderConfig
│   │   └── prompt.rs          # System prompts, prompt templates
│   │
│   ├── db/                    # Database layer
│   │   ├── mod.rs
│   │   ├── connection.rs      # DbConnection, DbConnectionConfig
│   │   ├── schema.rs          # DatabaseSchema, SchemaTable, ColumnInfo
│   │   ├── query.rs           # QueryResult, result formatting
│   │   └── executor.rs        # Query execution logic
│   │
│   ├── tools/                 # Tool system
│   │   ├── mod.rs
│   │   ├── registry.rs        # ToolRegistry
│   │   ├── trait.rs           # Tool trait definition
│   │   ├── executor.rs        # Tool execution logic
│   │   │
│   │   ├── built_in/
│   │   │   ├── mod.rs
│   │   │   ├── query_tool.rs  # execute_query tool
│   │   │   ├── schema_tool.rs # get_schema, list_tables tools
│   │   │   └── explain_tool.rs # explain_query tool
│   │   │
│   │   └── security/
│   │       ├── mod.rs
│   │       ├── validator.rs   # SafetyValidator
│   │       ├── blacklist.rs   # SQL blacklist patterns
│   │       └── permissions.rs # Permission checks
│   │
│   ├── tui/                   # Terminal UI
│   │   ├── mod.rs
│   │   ├── app.rs             # PostgresAgentTui, AppState
│   │   ├── views/
│   │   │   ├── mod.rs
│   │   │   ├── chat.rs        # Conversation view
│   │   │   ├── results.rs     # Query results view
│   │   │   ├── schema.rs      # Schema browser
│   │   │   └── settings.rs    # Settings view
│   │   ├── components/
│   │   │   ├── mod.rs
│   │   │   ├── input.rs       # Text input component
│   │   │   ├── status_bar.rs  # Status bar
│   │   │   ├── table.rs       # Result table
│   │   │   └── pagination.rs  # Pagination control
│   │   └── events/
│   │       ├── mod.rs
│   │       ├── handler.rs      # EventHandler
│   │       └── key.rs          # Key bindings
│   │
│   ├── config/                # Configuration management
│   │   ├── mod.rs
│   │   ├── app_config.rs      # Config struct
│   │   ├── database.rs         # DatabaseProfile
│   │   ├── llm.rs             # LlmConfig
│   │   ├── safety.rs          # SafetyConfig
│   │   └── loader.rs          # Config loading, file watching
│   │
│   ├── safety/                # Safety & audit layer
│   │   ├── mod.rs
│   │   ├── validator.rs       # SafetyValidator
│   │   ├── audit.rs           # AuditLogger, AuditEvent
│   │   ├── confirmation.rs    # Confirmation workflow
│   │   └── pii.rs            # PII detection
│   │
│   ├── cli/                   # CLI commands
│   │   ├── mod.rs
│   │   ├── commands.rs        # CLI command definitions
│   │   └── args.rs           # ArgMatches handling
│   │
│   └── util/                   # Utilities
│       ├── mod.rs
│       ├── logger.rs          # Logging setup
│       ├── metrics.rs         # Application metrics
│       └── crypto.rs          # Secret handling
│
├── tests/
│   ├── unit/
│   │   ├── mod.rs
│   │   ├── agent_tests.rs
│   │   ├── context_tests.rs
│   │   ├── safety_tests.rs
│   │   └── config_tests.rs
│   │
│   ├── integration/
│   │   ├── mod.rs
│   │   ├── agent_db_tests.rs
│   │   └── llm_tests.rs
│   │
│   └── e2e/
│       ├── mod.rs
│       ├── cli_tests.rs
│       └── tui_tests.rs
│
├── fixtures/
│   ├── config/
│   │   ├── valid.yaml
│   │   └── invalid.yaml
│   └── schemas/
│       ├── sample_schema.sql
│       └── test_data.sql
│
├── scripts/
│   ├── setup_dev.sh
│   ├── run_tests.sh
│   └── release_build.sh
│
└── docs/
    ├── architecture.md
    ├── user_guide.md
    ├── api/
    │   └── README.md
    └── troubleshooting.md
```

---

## 3. Cargo Workspace Configuration

### 3.1 Root Cargo.toml

```toml
[workspace]
members = [
    "crates/core",
    "crates/llm",
    "crates/db",
    "crates/tools",
    "crates/tui",
    "crates/config",
    "crates/safety",
    "crates/cli",
    "crates/util",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.93.0"
authors = ["PostgreSQL Agent Contributors"]

[workspace.dependencies]
tokio = { version = "1.49", features = ["rt-multi-thread", "macros", "sync", "tracing"] }
sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "postgres", "json"] }
async-openai = "0.32.4"
ratatui = { version = "0.30.0", features = ["crossterm", "serde"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
anyhow = "1.0.100"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
secrecy = "0.8"
async-trait = "0.1"
```

### 3.2 Individual Crate Configurations

#### crates/core/Cargo.toml

```toml
[package]
name = "postgres-agent-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
anyhow.workspace = true
tracing.workspace = true
secrecy.workspace = true
async-trait.workspace = true

# Internal
postgres-agent-llm = { path = "../llm" }
postgres-agent-db = { path = "../db" }
postgres-agent-tools = { path = "../tools" }
postgres-agent-safety = { path = "../safety" }
postgres-agent-config = { path = "../config" }
```

#### crates/llm/Cargo.toml

```toml
[package]
name = "postgres-agent-llm"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
tokio.workspace = true
async-openai.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
tracing.workspace = true
secrecy.workspace = true

# Internal
postgres-agent-util = { path = "../util" }
```

#### crates/db/Cargo.toml

```toml
[package]
name = "postgres-agent-db"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
tokio.workspace = true
sqlx.workspace = true
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
anyhow.workspace = true
tracing.workspace = true
secrecy.workspace = true

# Internal
postgres-agent-util = { path = "../util" }
postgres-agent-config = { path = "../config" }
```

---

## 4. Implementation Tasks by Phase

### Phase 1: MVP (Weeks 1-4)

#### Week 1: Project Setup & Foundation

| Task | Description | Deliverables |
|------|-------------|--------------|
| 1.1 | Initialize workspace structure | Cargo.toml, workspace setup |
| 1.2 | Configure toolchain and CI | rust-toolchain.toml, GitHub Actions |
| 1.3 | Set up logging infrastructure | logger.rs, tracing initialization |
| 1.4 | Create utility module | Secret handling, common types |
| 1.5 | Set up configuration loading | Config loader, .env support |

**Files to create:**
- `Cargo.toml` (workspace)
- `rust-toolchain.toml`
- `.env.example`
- `.github/workflows/ci.yml`
- `src/util/logger.rs`
- `src/util/mod.rs`
- `src/util/crypto.rs`

**Code structure:**
```rust
// src/util/logger.rs
pub fn setup_logger(level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_env("PG_AGENT_LOG_LEVEL")
        .unwrap_or_else(|_| EnvFilter::new(level));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(env_filter)
        .with_line_number(true)
        .with_thread_names(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
```

---

#### Week 2: Core Agent & Context

| Task | Description | Deliverables |
|------|-------------|--------------|
| 2.1 | Implement AgentContext | Message history, turn context |
| 2.2 | Implement AgentConfig | Safety levels, iteration limits |
| 2.3 | Implement PostgresAgent core | ReAct loop skeleton |
| 2.4 | Implement AgentDecision types | Reasoning, ToolCall, FinalAnswer |
| 2.5 | Implement error types | AgentError enum |

**Files to create:**
- `src/core/mod.rs`
- `src/core/context.rs`
- `src/core/agent.rs`
- `src/core/decision.rs`
- `src/core/error.rs`

**Key structures:**
```rust
// src/core/agent.rs
pub struct PostgresAgent {
    llm_client: Box<dyn LlmClient>,
    context: AgentContext,
    tools: ToolRegistry,
    config: AgentConfig,
}

impl PostgresAgent {
    pub async fn run(&mut self, query: &str) -> Result<AgentResponse, AgentError> {
        // ReAct loop implementation
    }
}
```

**Tests to write:**
- Context message history management
- Context window enforcement
- Decision parsing/serialization

---

#### Week 3: LLM Integration

| Task | Description | Deliverables |
|------|-------------|--------------|
| 3.1 | Define LlmClient trait | Provider abstraction |
| 3.2 | Implement OpenAiProvider | async-openai wrapper |
| 3.3 | Implement message conversion | To/from OpenAI format |
| 3.4 | Implement tool call parsing | Response parsing |
| 3.5 | Create system prompts | Prompt templates |

**Files to create:**
- `src/llm/mod.rs`
- `src/llm/client.rs`
- `src/llm/openai.rs`
- `src/llm/provider.rs`
- `src/llm/prompt.rs`

**Key structures:**
```rust
// src/llm/client.rs
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate_decision(
        &self,
        context: &AgentContext,
    ) -> Result<AgentDecision, LlmError>;

    async fn generate_structured<T: serde::de::DeserializeOwned + Debug>(
        &self,
        prompt: &str,
        schema: &T,
    ) -> Result<T, LlmError>;

    fn provider_info(&self) -> ProviderInfo;
}
```

**Tests to write:**
- Mock LlmClient for testing
- Tool definition serialization
- Prompt template rendering

---

#### Week 4: Database Layer

| Task | Description | Deliverables |
|------|-------------|--------------|
| 4.1 | Implement DbConnection | Connection pool management |
| 4.2 | Implement schema introspection | Table/column/foreign key queries |
| 4.3 | Implement query execution | SELECT query running |
| 4.4 | Implement result formatting | JSON/table output |
| 4.5 | Implement QueryResult types | Row data structures |

**Files to create:**
- `src/db/mod.rs`
- `src/db/connection.rs`
- `src/db/schema.rs`
- `src/db/query.rs`
- `src/db/executor.rs`

**Key structures:**
```rust
// src/db/connection.rs
pub struct DbConnection {
    pool: sqlx::PgPool,
    config: DbConnectionConfig,
}

impl DbConnection {
    pub async fn execute_query(&self, sql: &str) -> Result<QueryResult, DbError> {
        // SELECT-only validation
        // Timeout enforcement
        // Result serialization
    }

    pub async fn get_schema(&self, table_filter: Option<&str>) -> Result<DatabaseSchema, DbError> {
        // information_schema queries
    }
}
```

**Tests to write:**
- Connection pool management
- Schema introspection (mock DB)
- Query result serialization
- Non-SELECT query rejection

---

### Phase 2: Core Features (Weeks 5-8)

#### Week 5: Tool System

| Task | Description | Deliverables |
|------|-------------|--------------|
| 5.1 | Define Tool trait | Tool interface |
| 5.2 | Implement ToolRegistry | Tool registration, lookup |
| 5.3 | Implement built-in tools | query_database, get_schema |
| 5.4 | Implement tool context | Safety context, timeouts |
| 5.5 | Implement parallel execution | JoinSet for parallel calls |

**Files to create:**
- `src/tools/mod.rs`
- `src/tools/trait.rs`
- `src/tools/registry.rs`
- `src/tools/executor.rs`
- `src/tools/built_in/mod.rs`
- `src/tools/built_in/query_tool.rs`
- `src/tools/built_in/schema_tool.rs`
- `src/tools/built_in/explain_tool.rs`

**Key structures:**
```rust
// src/tools/trait.rs
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    const NAME: &'static str;

    fn definition(&self) -> ToolDefinition;
    fn check_permissions(&self, ctx: &ToolContext) -> Result<(), ToolError>;
    async fn execute(&self, args: &serde_json::Value) -> Result<serde_json::Value, ToolError>;
}

// src/tools/built_in/query_tool.rs
pub struct QueryTool {
    db: DbConnection,
    safety: SafetyContext,
}

#[async_trait::async_trait]
impl Tool for QueryTool {
    const NAME: &'static str = "execute_query";
    // ...
}
```

---

#### Week 6: Safety & Validation

| Task | Description | Deliverables |
|------|-------------|--------------|
| 6.1 | Implement SafetyValidator | Blacklist pattern matching |
| 6.2 | Implement SQL classification | SELECT vs DML/DDL detection |
| 6.3 | Implement safety levels | ReadOnly, Balanced, Permissive |
| 6.4 | Implement confirmation workflow | Simple, Typed, Admin approval |
| 6.5 | Implement PII detection | Regex-based PII patterns |

**Files to create:**
- `src/safety/mod.rs`
- `src/safety/validator.rs`
- `src/safety/blacklist.rs`
- `src/safety/confirmation.rs`
- `src/safety/permissions.rs`
- `src/safety/pii.rs`

**Key structures:**
```rust
// src/safety/validator.rs
pub struct SafetyValidator {
    blacklist: Vec<Regex>,
    allowed_ops: HashMap<SafetyLevel, HashSet<OperationType>>,
    pii_detector: PiiDetector,
}

impl SafetyValidator {
    pub fn validate(&self, sql: &str, ctx: &SafetyContext) -> Result<ValidationResult, SafetyError> {
        // Check blacklisted patterns
        // Classify operation type
        // Verify permissions
        // Check read-only mode
    }
}
```

**Tests to write:**
- Blacklist pattern matching
- Operation classification
- Safety level enforcement
- PII detection and redaction

---

#### Week 7: Configuration System

| Task | Description | Deliverables |
|------|-------------|--------------|
| 7.1 | Implement AppConfig | Full configuration struct |
| 7.2 | Implement DatabaseProfile | Database connection config |
| 7.3 | Implement LlmConfig | LLM provider config |
| 7.4 | Implement ConfigLoader | YAML loading, env overrides |
| 7.5 | Implement config validation | Required fields, defaults |

**Files to create:**
- `src/config/mod.rs`
- `src/config/app_config.rs`
- `src/config/database.rs`
- `src/config/llm.rs`
- `src/config/safety.rs`
- `src/config/loader.rs`

**Key structures:**
```rust
// src/config/app_config.rs
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub databases: Vec<DatabaseProfile>,
    #[serde(default)]
    pub agent: AgentConfig,
    #[serde(default)]
    pub safety: SafetyConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

// src/config/loader.rs
pub struct ConfigLoader {
    config_path: PathBuf,
    #[allow(dead_code)]
    watcher: FileWatcher,
}

impl ConfigLoader {
    pub async fn load(&self) -> Result<AppConfig, ConfigError>;
    pub fn try_load(&self) -> Result<AppConfig, ConfigError>;
}
```

---

#### Week 8: TUI Basics & CLI

| Task | Description | Deliverables |
|------|-------------|--------------|
| 8.1 | Implement ratatui app skeleton | Main loop, event handling |
| 8.2 | Implement chat view | Message display, input area |
| 8.3 | Implement status bar | Connection status, safety level |
| 8.4 | Implement command palette | Quick actions |
| 8.5 | Implement CLI interface | Clap commands |

**Files to create:**
- `src/tui/mod.rs`
- `src/tui/app.rs`
- `src/tui/views/chat.rs`
- `src/tui/views/results.rs`
- `src/tui/components/input.rs`
- `src/tui/components/status_bar.rs`
- `src/tui/events/handler.rs`
- `src/cli/mod.rs`
- `src/cli/commands.rs`
- `src/cli/args.rs`

**Key structures:**
```rust
// src/tui/app.rs
pub struct PostgresAgentTui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: AppState,
    events: EventHandler,
    agent: PostgresAgent,
}

impl PostgresAgentTui {
    pub async fn run(&mut self) -> Result<(), TuiError> {
        self.terminal.clear()?;
        loop {
            self.terminal.draw(|f| self.render(f))?;
            match self.events.next().await {
                Event::Key(key) => self.handle_key_event(key).await?,
                Event::Resize(width, height) => self.state.size = Size { width, height },
            }
        }
    }
}
```

---

### Phase 3: Advanced Features (Weeks 9-12)

#### Week 9: Audit & Logging

| Task | Description | Deliverables |
|------|-------------|--------------|
| 9.1 | Implement AuditLogger | Log storage, formatting |
| 9.2 | Define AuditEvent types | Query, SchemaChange, SafetyViolation |
| 9.3 | Implement audit sinks | File, stdout, remote |
| 9.4 | Integrate audit logging | Log all agent actions |
| 9.5 | Implement query history | Persistent conversation history |

**Files to create:**
- `src/safety/audit.rs`
- `src/safety/events.rs`
- `src/safety/sinks.rs`

**Key structures:**
```rust
// src/safety/audit.rs
pub struct AuditLogger {
    sink: AuditSink,
    formatter: AuditFormatter,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum AuditEvent {
    Query {
        timestamp: DateTime<Utc>,
        user: String,
        database: String,
        query: String,
        success: bool,
        duration_ms: u64,
    },
    SchemaChange {
        timestamp: DateTime<Utc>,
        user: String,
        database: String,
        operation: String,
        sql: String,
        approved: bool,
    },
    SafetyViolation {
        timestamp: DateTime<Utc>,
        user: String,
        query: String,
        reason: String,
    },
}
```

---

#### Week 10: Export & Data Formatting

| Task | Description | Deliverables |
|------|-------------|--------------|
| 10.1 | Implement CSV export | Export query results |
| 10.2 | Implement JSON export | Structured output |
| 10.3 | Implement table rendering | Pretty-printed tables |
| 10.4 | Implement pagination | Large result sets |
| 10.5 | Implement EXPLAIN analysis | Query plan display |

**Files to create:**
- `src/tui/components/table.rs`
- `src/tui/components/pagination.rs`
- `src/db/explain.rs`
- `src/util/export.rs`

---

#### Week 11: Schema Browser & Index Advisor

| Task | Description | Deliverables |
|------|-------------|--------------|
| 11.1 | Implement schema view | Table list, column details |
| 11.2 | Implement FK visualization | Relationship diagram |
| 11.3 | Implement index recommendations | Missing index detection |
| 11.4 | Implement EXPLAIN ANALYZE | Performance analysis |
| 11.5 | Connect schema tool to TUI | Schema browser UI |

**Files to create:**
- `src/tui/views/schema.rs`
- `src/db/index_analysis.rs`
- `src/db/explain.rs`

---

#### Week 12: Integration & Polish

| Task | Description | Deliverables |
|------|-------------|--------------|
| 12.1 | Integration testing | Full workflow tests |
| 12.2 | Error message improvements | User-friendly errors |
| 12.3 | Performance optimization | Caching, connection pooling |
| 12.4 | Documentation | User guide, API docs |
| 12.5 | Release preparation | Version bump, CHANGELOG |

---

## 5. Tool Implementation Details

### 5.1 Core Tools

| Tool Name | Description | Parameters |
|-----------|-------------|------------|
| `execute_query` | Execute a SQL SELECT query | `{ sql: string }` |
| `get_schema` | Get full database schema | `{ filter?: string }` |
| `list_tables` | List all tables | `{}` |
| `describe_table` | Describe specific table | `{ table_name: string }` |
| `explain_query` | Get query execution plan | `{ sql: string }` |

### 5.2 Tool Definition JSON Schema

```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "description": { "type": "string" },
    "parameters": {
      "type": "object",
      "properties": {
        "type": { "type": "string", "enum": ["object"] },
        "properties": {
          "sql": { "type": "string" }
        },
        "required": ["sql"]
      }
    }
  }
}
```

---

## 6. Error Handling Strategy

### 6.1 Error Type Hierarchy

```rust
// src/core/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("agent error: {source}")]
    Agent { #[from] source: AgentError },

    #[error("database error: {source}")]
    Database { #[from] source: DbError },

    #[error("LLM error: {source}")]
    Llm { #[from] source: LlmError },

    #[error("configuration error: {source}")]
    Configuration { #[from] source: ConfigError },

    #[error("safety violation: {reason}")]
    Safety { reason: String },

    #[error("TUI error: {source}")]
    Tui { #[from] source: TuiError },
}
```

### 6.2 Retry Logic

```rust
// Retry configuration for transient errors
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl RetryConfig {
    pub fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}
```

---

## 7. Testing Strategy

### 7.1 Testing Pyramid

| Level | Coverage | Tools |
|-------|----------|-------|
| Unit | 70% | `#[test]`, `rstest`, `proptest` |
| Integration | 20% | Module tests with mocks |
| E2E | 10% | CLI commands, TUI workflows |

### 7.2 Test Examples

**Unit Test - Safety Validator:**
```rust
#[cfg(test)]
mod tests {
    use rstest::*;
    use super::*;

    #[rstest]
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
}
```

**Integration Test - Agent + Mock LLM:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use postgres_agent_core::context::AgentContext;

    #[tokio::test]
    async fn test_agent_with_mock_llm() {
        let mock_llm = MockLlmClient::new()
            .with_decision(AgentDecision::ToolCall(ToolCall {
                name: "execute_query".to_string(),
                arguments: json!({ "sql": "SELECT 1" }),
                call_id: "test-1".to_string(),
            }));

        let agent = PostgresAgent::new(Box::new(mock_llm));
        let result = agent.run("Show me a test").await;
        assert!(result.is_ok());
    }
}
```

---

## 8. CI/CD Pipeline

### 8.1 GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
      - run: cargo fmt -- --check
      - run: cargo clippy --all-features -- -D warnings

  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.93.0
      - run: cargo test --all-features
      - run: cargo test --all-features --doc

  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release --all-features
      - uses: actions/upload-artifact@v4
        with:
          name: binaries
          path: target/release/pg_agent

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo tarpaulin --all-features --out Html
      - uses: actions/upload-artifact@v4
        with:
          name: coverage
          path: tarpaulin-report.html
```

---

## 9. Development Workflow

### 9.1 Setting Up Development Environment

```bash
# scripts/setup_dev.sh

#!/bin/bash
set -e

echo "Setting up PostgreSQL Agent development environment..."

# Install Rust if not installed
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

# Install sqlx-cli for compile-time checks
cargo install sqlx-cli --no-default-features --features postgres

# Set up PostgreSQL database for testing
docker run -d \
    --name postgres_agent_test \
    -e POSTGRES_PASSWORD=postgres \
    -p 5432:5432 \
    postgres:16

# Wait for PostgreSQL to be ready
until pg_isready -h localhost -p 5432 -U postgres; do
    echo "Waiting for PostgreSQL..."
    sleep 1
done

# Create test database
psql -h localhost -U postgres -c "CREATE DATABASE test_agent;"

# Copy environment file
cp .env.example .env

echo "Development environment ready!"
echo "Edit .env with your configuration."
```

### 9.2 Running Tests

```bash
# scripts/run_tests.sh

#!/bin/bash
set -e

# Run unit tests
echo "Running unit tests..."
cargo test --all-features -- --test-threads=4

# Run integration tests (requires PostgreSQL)
echo "Running integration tests..."
DATABASE_URL="postgres://postgres:postgres@localhost:5432/test_agent" \
    cargo test --test integration -- --test-threads=2

# Run doc tests
echo "Running documentation tests..."
cargo test --doc

# Run clippy
echo "Running clippy..."
cargo clippy --all-features -- -D warnings

echo "All tests passed!"
```

---

## 10. Milestone Checklist

### Milestone 1: MVP (End of Week 4)

- [ ] Project structure initialized
- [ ] Core agent with ReAct loop implemented
- [ ] LLM client (OpenAI) working
- [ ] Database connection and schema introspection
- [ ] Basic SQL generation and execution
- [ ] Simple TUI chat interface
- [ ] Safety level configuration
- [ ] All unit tests passing (>80% coverage)

### Milestone 2: Core Features (End of Week 8)

- [ ] Tool system with registry
- [ ] Built-in tools (query, schema, explain)
- [ ] Safety validator with blacklist
- [ ] Confirmation workflows
- [ ] Multi-database profile support
- [ ] Configuration file loading
- [ ] CLI interface
- [ ] Integration tests passing

### Milestone 3: Advanced Features (End of Week 12)

- [ ] Audit logging system
- [ ] Export to CSV/JSON
- [ ] Schema browser UI
- [ ] Index recommendations
- [ ] EXPLAIN analysis
- [ ] Full E2E tests passing
- [ ] Documentation complete
- [ ] Release build optimized

---

## 11. Dependencies Matrix

### Required Dependencies

| Crate | Version | Status | Notes |
|-------|---------|--------|-------|
| tokio | 1.x | Required | Async runtime |
| sqlx | 0.7+ | Required | PostgreSQL driver |
| async-openai | 0.2+ | Required | LLM provider |
| ratatui | 0.26+ | Required | Terminal UI |
| clap | 4.x | Required | CLI parsing |
| thiserror | 2.x | Required | Error types |
| anyhow | 1.x | Required | Error handling |
| tracing | 0.3 | Required | Logging |

### Optional Dependencies

| Crate | Feature | Phase | Purpose |
|-------|---------|-------|---------|
| secrecy | security | Week 6 | Secret management |
| regex | validation | Week 6 | Pattern matching |
| csv | export | Week 10 | CSV export |
| chrono | time | Week 9 | Timestamps |
| refinery | migrations | Future | DB migrations |

---

## 12. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| LLM API rate limits | High | Retry logic, caching |
| Complex schema handling | Medium | Hierarchical retrieval |
| SQL injection via LLM | High | Validation, read-only mode |
| Large result sets | Medium | Pagination, streaming |
| TUI compatibility | Low | Fallback to CLI |
| Performance with many tables | Medium | Schema caching |

---

## 13. References

- Technical Specification: `0005_postgres_agent_spec.md`
- Research Document: `0002_research.md`
- sqlx Documentation: https://docs.rs/sqlx
- ratatui Documentation: https://ratatui.rs/
- async-openai Documentation: https://docs.rs/async-openai
- ReAct Paper: https://arxiv.org/abs/2210.03629
