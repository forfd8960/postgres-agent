# Postgres AI Agent — Product Requirements Document (PRD)

## 1. Summary
Build a Rust-based AI agent that enables users to manage and query PostgreSQL databases using natural language through a terminal UI. The agent translates intent into safe SQL operations, executes them via PostgreSQL, and returns results with context-aware guidance.

## 2. Goals
- Provide a natural-language interface for common PostgreSQL tasks.
- Support multiple database configurations and easy switching.
- Offer a responsive, interactive TUI for iterative exploration.
- Ensure safe execution with permissions, confirmations, and auditing.
- Maintain conversational context for multi-step workflows.

## 3. Non-Goals
- Building a full GUI or web UI.
- Replacing DBAs or advanced query optimization tools.
- Supporting non-PostgreSQL databases in v1.

## 4. Target Users & Personas
- **Developer**: Needs quick queries, schema discovery, and data manipulation without writing SQL.
- **Analyst**: Runs ad hoc reads, filters, and aggregates with minimal SQL knowledge.
- **DBA**: Performs controlled maintenance tasks with explicit confirmations and audit logs.

## 5. User Stories
- As a developer, I can ask “show latest 20 orders” and get results.
- As an analyst, I can ask “total revenue last month by region” and receive a table.
- As a DBA, I can request “create index on orders(created_at)” and review a confirmation prompt.
- As a user, I can switch between multiple configured databases.
- As a user, I can export results to CSV or JSON.

## 6. Functional Requirements

### 6.1 Natural Language Querying
- Interpret plain-English requests and generate SQL.
- Support `SELECT` queries, filters, groupings, joins, and limits.
- Provide SQL preview before execution when configured to do so.

### 6.2 Database Management
- Run DML operations (`INSERT`, `UPDATE`, `DELETE`) with confirmation.
- Generate migrations from user intent (e.g., add column, create table).
- Create and drop indexes with validation.
- Analyze performance using `EXPLAIN` / `EXPLAIN ANALYZE`.
- Backup and restore operations with explicit confirmations.

### 6.3 Multi-Database Configuration
- Load multiple DB profiles from config file.
- Allow switching active DB during session.
- Display active DB context in the UI.

### 6.4 TUI (Terminal UI)
- Interactive chat-like interface.
- Results pane with pagination and scrolling.
- Command palette for actions (switch DB, export, history, help).

### 6.5 Context & Memory
- Maintain session context, including recent queries and schema info.
- Allow user to clear or pin context.

### 6.6 Exporting
- Export query results to CSV and JSON.
- Allow user to specify output path.

### 6.7 Observability & Logging
- Log operations with timestamps and outcomes.
- Provide audit trail for changes.

## 7. Non-Functional Requirements
- **Performance**: Queries should return with minimal latency; UI must remain responsive.
- **Reliability**: Graceful handling of connection issues and API errors.
- **Security**: Respect DB permissions, prevent unsafe operations without confirmation.
- **Privacy**: Never store sensitive data by default; configurable redaction.
- **Portability**: Runs on macOS, Linux, and Windows terminals.

## 8. Configuration Requirements
- Configurable model, base URL, API key, and timeout.
- Configurable DB profiles (host, port, user, db, SSL).
- Configurable safety levels (read-only mode, require confirmations).

## 9. Safety & Permissions
- Default to read-only unless explicitly enabled.
- Require confirmation for DML and DDL.
- Provide SQL preview before execution.
- Detect destructive queries and require extra confirmation.

## 10. Technical Stack
- **Rust**
- **clap + ratatui** for CLI/TUI
- **sqlx** for PostgreSQL access
- **tokio** for async runtime
- **serde** for serialization
- **async-openai** for LLM integration
- **anyhow** for error handling
- **tracing** for logging
- **dotenvy** for environment loading

## 11. Dependencies & Integrations
- PostgreSQL server (user-provided)
- OpenAI-compatible API endpoint

## 12. UX Requirements
- Clear status bar (active DB, model, safety mode)
- Results displayed in tables with pagination
- Explicit confirmations for destructive actions
- Helpful errors with actionable steps

## 13. Error Handling
- Retry policy for transient API errors.
- DB connection failures should show actionable messages.
- LLM parsing errors should fall back to a prompt for clarification.

## 14. Acceptance Criteria
- User can connect to at least one configured DB and run natural-language queries.
- User can switch between two DB profiles without restarting.
- Agent requires confirmation before executing DML/DDL.
- Query results can be exported to CSV and JSON.
- TUI remains responsive during long-running queries.

## 15. Milestones (Proposed)
1. **MVP**: NL → SQL read-only queries + TUI + single DB config.
2. **Core**: Multi-DB, confirmations, export, audit logging.
3. **Advanced**: Migrations, indexing, performance analysis.

## 16. Risks & Mitigations
- **Incorrect SQL generation** → SQL preview + confirmations + read-only mode.
- **Large result sets** → pagination + export.
- **Model latency** → async execution + UI loading states.

## 17. Open Questions
- Should we allow offline/local model support in v1?
- What is the default safe mode (read-only vs read-write)?
- What schema caching strategy is preferred?
