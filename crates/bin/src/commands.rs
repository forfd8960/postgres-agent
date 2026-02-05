//! Command implementations for PostgreSQL Agent CLI.
//!
//! Contains all the command handler functions for the CLI.

use anyhow::{bail, Context, Result};
use postgres_agent_config::safety::SafetyLevel as ConfigSafetyLevel;
use postgres_agent_config::{AppConfig, ConfigLoader, DatabaseProfile};
use postgres_agent_core::agent::{AgentConfig, AgentResponse, PostgresAgent};
use postgres_agent_core::agent::SafetyLevel as CoreSafetyLevel;
use postgres_agent_db::executor::QueryResult;
use postgres_agent_db::{DbConnection, DbConnectionConfig, QueryExecutor};
use postgres_agent_llm::client::LlmClient;
use postgres_agent_llm::openai::OpenAiProvider;
use postgres_agent_llm::provider::ProviderConfig;
use postgres_agent_tools::ToolContext;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tracing::error;

use postgres_agent_cli::OutputFormat;

// ============================================================================
// Command Handlers
// ============================================================================

/// Run a single query using the agent.
pub async fn run_query(
    query: &str,
    config_path: &str,
    profile_name: &str,
    output_format: &str,
    safety_level: Option<&str>,
    no_confirm: bool,
    quiet: bool,
) -> Result<()> {
    let start = std::time::Instant::now();

    // Load configuration
    let config = load_config(config_path).await?;

    // Get database profile
    let profile = get_profile(&config, profile_name)?;

    // Create database connection
    let db = create_connection(&profile).await?;

    // Create LLM client
    let llm_client = create_llm_client(&config)?;

    // Create agent with tools
    let mut agent = create_agent(llm_client, &db, &config, safety_level, no_confirm)?;

    // Run the agent
    let response = agent.run(query).await;

    let duration_ms = start.elapsed().as_millis();

    // Handle result
    match response {
        Ok(agent_response) => {
            if !quiet {
                println!("\n{}", "=".repeat(60));
            }

            let format = OutputFormat::from_str(output_format).unwrap_or(OutputFormat::Table);

            if !quiet {
                println!("Query: {}", query);
                println!("Duration: {}ms", duration_ms);
                println!("Iterations: {}", agent_response.iterations);
                if let Some(sql) = &agent_response.executed_sql {
                    println!("SQL: {}", sql);
                }
            }

            print_response(&agent_response, format);

            if !quiet {
                println!("{}", "=".repeat(60));
            }

            Ok(())
        }
        Err(e) => {
            error!("Query failed: {}", e);
            bail!("Agent error: {}", e);
        }
    }
}

/// Run interactive TUI mode.
pub async fn run_interactive(
    config_path: &str,
    profile_name: &str,
    safety_level: Option<&str>,
    no_confirm: bool,
) -> Result<()> {
    println!("Starting interactive mode...");
    println!("Profile: {}", profile_name);
    println!("(TUI mode - basic CLI REPL active)\n");

    // Load configuration
    let config = load_config(config_path).await?;
    let profile = get_profile(&config, profile_name)?;
    let db = create_connection(&profile).await?;
    let llm_client = create_llm_client(&config)?;
    let mut agent = create_agent(llm_client, &db, &config, safety_level, no_confirm)?;

    println!("PostgreSQL Agent Interactive Mode");
    println!("Type 'exit' or 'quit' to exit.\n");

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        print!("> ");
        std::io::stdout().flush()?;

        input.clear();
        stdin.read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("\\help") || input.eq_ignore_ascii_case("\\h") {
            print_interactive_help();
            continue;
        }

        match agent.run(input).await {
            Ok(response) => {
                println!("\n{}", response.answer);
                if let Some(sql) = &response.executed_sql {
                    println!("[SQL: {}]", sql);
                }
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        println!();
    }

    Ok(())
}

/// Execute SQL from files.
pub async fn execute_files(
    files: &[String],
    config_path: &str,
    profile_name: &str,
    output_format: &str,
    quiet: bool,
) -> Result<()> {
    let config = load_config(config_path).await?;
    let profile = get_profile(&config, profile_name)?;
    let db = create_connection(&profile).await?;
    let executor = QueryExecutor::new(db);

    let format = OutputFormat::from_str(output_format).unwrap_or(OutputFormat::Table);

    for file in files {
        let path = PathBuf::from(file);
        if !path.exists() {
            bail!("File not found: {}", file);
        }

        let sql = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file: {}", file))?;

        if !quiet {
            println!("Executing: {}", file);
        }

        match executor.execute_query(&sql).await {
            Ok(result) => {
                if !quiet {
                    println!("Rows: {:?}", result.row_count);
                    if let Some(time) = result.execution_time_ms {
                        println!("Time: {}ms", time);
                    }
                }
                print_query_result(&result, format);
            }
            Err(e) => {
                bail!("Error executing {}: {}", file, e);
            }
        }
    }

    Ok(())
}

/// List available database profiles.
pub async fn list_profiles(config_path: &str) -> Result<()> {
    let config = load_config(config_path).await?;

    println!("\nDatabase Profiles");
    println!("{}\n", "=".repeat(40));

    if config.databases.is_empty() {
        println!("No profiles configured.");
        println!("\nAdd profiles to your config file:");
        println!("  [[databases]]");
        println!("  name = \"default\"");
        println!("  url = \"postgres://user:pass@localhost:5432/db\"");
        return Ok(());
    }

    for profile in &config.databases {
        println!("- [{}] {}", profile.name, mask_url(&profile.url));
    }

    Ok(())
}

/// Show current configuration.
pub async fn show_config(config_path: &str, _effective: bool) -> Result<()> {
    let config = load_config(config_path).await?;

    println!("\nConfiguration");
    println!("{}\n", "=".repeat(50));

    // LLM Configuration
    println!("LLM:");
    println!("  Provider: {}", config.llm.provider);
    println!("  Model: {}", config.llm.model);
    if let Some(ref url) = config.llm.base_url {
        println!("  Base URL: {}", url);
    }
    println!("  Temperature: {}", config.llm.temperature);
    println!("  Max tokens: {}", config.llm.max_tokens);
    println!();

    // Agent Configuration
    println!("Agent:");
    println!("  Max iterations: {}", config.agent.max_iterations);
    println!();

    // Safety Configuration
    println!("Safety:");
    println!("  Safety level: {:?}", config.safety.safety_level);
    println!("  Require confirmation: {}", config.safety.require_confirmation);
    println!();

    // Databases
    println!("Databases: {} configured", config.databases.len());
    for db in &config.databases {
        println!("  - {}", db.name);
    }

    Ok(())
}

/// Show database schema.
pub async fn show_schema(
    config_path: &str,
    profile_name: &str,
    table_filter: Option<&str>,
) -> Result<()> {
    let config = load_config(config_path).await?;
    let profile = get_profile(&config, profile_name)?;
    let db = create_connection(&profile).await?;
    let executor = QueryExecutor::new(db);

    let schema = executor
        .get_schema(table_filter)
        .await
        .context("Failed to get schema")?;

    println!("\nDatabase Schema");
    println!("{}\n", "=".repeat(50));

    if schema.tables.is_empty() {
        println!("No tables found.");
        return Ok(());
    }

    for table in &schema.tables {
        println!("- {}.{}", table.table_schema, table.table_name);
        if let Some(columns) = schema.columns.get(&table.table_name) {
            for col in columns {
                println!("    {} ({})", col.column_name, col.data_type);
            }
        }
    }

    Ok(())
}

/// Run system doctor check.
pub async fn run_doctor(config_path: &str) -> Result<()> {
    println!("\nPostgreSQL Agent System Check");
    println!("{}\n", "=".repeat(50));

    let mut checks_passed = 0;
    let mut checks_total = 0;

    // Check configuration file
    checks_total += 1;
    let config_exists = PathBuf::from(config_path).exists();
    print_check("Config file", config_exists);
    if config_exists {
        checks_passed += 1;
    }

    // Check configuration
    checks_total += 1;
    match load_config(config_path).await {
        Ok(config) => {
            print_check("Configuration", true);
            checks_passed += 1;

            // Check LLM configuration
            checks_total += 1;
            let llm_ok = !config.llm.model.is_empty() && config.llm.max_tokens > 0;
            print_check("LLM configuration", llm_ok);
            if llm_ok {
                checks_passed += 1;
            }

            // Check database configuration
            checks_total += 1;
            let db_ok = !config.databases.is_empty()
                && config.databases.iter().all(|p| !p.name.is_empty());
            print_check("Database configuration", db_ok);
            if db_ok {
                checks_passed += 1;
            }
        }
        Err(e) => {
            print_check("Configuration", false);
            println!("    Error: {}", e);
        }
    }

    println!("\n{} {}/{} checks passed", "Result:", checks_passed, checks_total);

    if checks_passed == checks_total {
        println!("\nSystem is ready for use!");
    } else {
        println!("\nSome checks failed. Review the output above.");
    }

    Ok(())
}

/// Print interactive mode help.
fn print_interactive_help() {
    println!("\nAvailable commands:");
    println!("  \\q, \\quit, exit  - Exit interactive mode");
    println!();
    println!("Tips:");
    println!("  - Type natural language queries");
    println!("  - Ask about your database schema");
    println!("  - Request data analysis or aggregations");
    println!();
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Load configuration from file.
async fn load_config(config_path: &str) -> Result<AppConfig> {
    let mut loader = ConfigLoader::new(config_path);
    loader.try_load().with_context(|| {
        format!("Failed to load configuration from '{}'", config_path)
    })
}

/// Get database profile by name.
fn get_profile(config: &AppConfig, name: &str) -> Result<DatabaseProfile> {
    config
        .databases
        .iter()
        .find(|p| p.name == name)
        .or(config.databases.first())
        .cloned()
        .with_context(|| format!("Database profile '{}' not found", name))
}

/// Convert string to SslMode enum.
fn parse_ssl_mode(s: &str) -> postgres_agent_db::SslMode {
    match s.to_lowercase().as_str() {
        "disable" | "disabled" => postgres_agent_db::SslMode::Disable,
        "require" | "required" => postgres_agent_db::SslMode::Require,
        "prefer" | "preferred" | _ => postgres_agent_db::SslMode::Prefer,
    }
}

/// Create database connection.
async fn create_connection(profile: &DatabaseProfile) -> Result<DbConnection> {
    let db_config = DbConnectionConfig {
        url: profile.url.clone(),
        host: None,
        port: None,
        username: None,
        password: None,
        database: None,
        max_connections: 5,
        min_idle_connections: 1,
        connect_timeout: profile.connect_timeout,
        query_timeout: 60,
        ssl_mode: parse_ssl_mode(&profile.ssl_mode),
    };

    DbConnection::new(&db_config).await.with_context(|| {
        format!("Failed to connect to database '{}'", profile.name)
    })
}

/// Create LLM client from configuration.
fn create_llm_client(config: &AppConfig) -> Result<OpenAiProvider> {
    let api_key = config.llm.api_key.clone().ok_or_else(|| {
        anyhow::anyhow!("API key not configured")
    })?;

    let provider_config = ProviderConfig {
        provider_type: config.llm.provider.clone(),
        base_url: config.llm.base_url.clone(),
        api_key: Some(api_key),
        model: config.llm.model.clone(),
        temperature: config.llm.temperature,
        max_tokens: config.llm.max_tokens,
    };

    Ok(OpenAiProvider::new(provider_config))
}

/// Create agent with tools.
fn create_agent<C: LlmClient>(
    llm_client: C,
    _db: &DbConnection,
    config: &AppConfig,
    safety_level: Option<&str>,
    no_confirm: bool,
) -> Result<PostgresAgent<C>> {
    // Determine safety level
    let safety = match safety_level {
        Some(s) => parse_safety_level(s),
        None => map_safety_level(config.safety.safety_level),
    };

    // Create tool context with timeout
    let tool_context = ToolContext::with_timeout(Duration::from_secs(30));

    // Create agent config - use default values for missing fields
    let agent_config = AgentConfig {
        max_iterations: config.agent.max_iterations,
        require_confirmation: !no_confirm,
        safety_level: safety,
        timeout_seconds: 30,
        verbose_reasoning: false,
    };

    // Create agent
    let mut agent = PostgresAgent::with_config(Box::new(llm_client), agent_config);
    agent.set_tool_context(tool_context);

    Ok(agent)
}

/// Parse safety level string to core SafetyLevel enum.
fn parse_safety_level(s: &str) -> CoreSafetyLevel {
    match s.to_lowercase().as_str() {
        "read_only" | "readonly" => CoreSafetyLevel::ReadOnly,
        "balanced" => CoreSafetyLevel::Balanced,
        "permissive" => CoreSafetyLevel::Permissive,
        _ => CoreSafetyLevel::Balanced,
    }
}

/// Map config safety level to core safety level.
fn map_safety_level(s: ConfigSafetyLevel) -> CoreSafetyLevel {
    match s {
        ConfigSafetyLevel::ReadOnly => CoreSafetyLevel::ReadOnly,
        ConfigSafetyLevel::Balanced => CoreSafetyLevel::Balanced,
        ConfigSafetyLevel::Permissive => CoreSafetyLevel::Permissive,
    }
}

/// Print agent response based on format.
fn print_response(response: &AgentResponse, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "answer": response.answer,
                "success": response.success,
                "iterations": response.iterations,
                "executed_sql": response.executed_sql,
                "error": response.error,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
        }
        OutputFormat::Table | OutputFormat::Raw => {
            println!("{}", response.answer);
        }
        OutputFormat::Csv => {
            // Simple CSV output for answer
            println!("answer");
            println!("\"{}\"", response.answer.replace('"', "\"\""));
        }
    }
}

/// Print query result based on format.
fn print_query_result(result: &QueryResult, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "columns": result.columns,
                "rows": result.rows,
                "row_count": result.row_count,
                "execution_time_ms": result.execution_time_ms,
            });
            println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
        }
        OutputFormat::Table => {
            if result.columns.is_empty() {
                println!("No results.");
                return;
            }

            // Simple table output
            println!("{}", result.columns.join(" | "));
            println!("{}", "-".repeat(result.columns.iter().map(|c| c.len()).sum::<usize>()));

            for row in &result.rows {
                let row_str: Vec<String> = result
                    .columns
                    .iter()
                    .map(|col| {
                        row.get(col)
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string()
                    })
                    .collect();
                println!("{}", row_str.join(" | "));
            }
            println!("\n{} rows", result.row_count);
        }
        OutputFormat::Csv => {
            if !result.columns.is_empty() {
                println!("{}", result.columns.join(","));
                for row in &result.rows {
                    let row_str: Vec<String> = result
                        .columns
                        .iter()
                        .map(|col| {
                            let s = row.get(col).and_then(|v| v.as_str()).unwrap_or_default();
                            if s.contains(',') || s.contains('"') {
                                format!("\"{}\"", s.replace('"', "\"\""))
                            } else {
                                s.to_string()
                            }
                        })
                        .collect();
                    println!("{}", row_str.join(","));
                }
            }
        }
        OutputFormat::Raw => {
            println!("SQL executed successfully.");
            println!("Rows affected: {}", result.row_count);
        }
    }
}

/// Mask URL for display.
fn mask_url(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        let after_at = &url[at_pos + 1..];
        return format!("***@{}", after_at);
    }
    url.to_string()
}

/// Print a check result.
fn print_check<T: std::fmt::Display>(name: &str, result: T) {
    let status = "✓";
    println!("[{}] {}: {}", status, name, result);
}
