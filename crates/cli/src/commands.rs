//! CLI command implementations.
//!
//! This module provides implementations for CLI commands.

use anyhow::{Context, Result};
use postgres_agent_config::{AppConfig, ConfigLoader, DatabaseProfile};
use postgres_agent_core::context::Message;
use std::path::PathBuf;
use std::str::FromStr;

/// Output format for query results.
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    /// Table format (default).
    Table,
    /// JSON format.
    Json,
    /// CSV format.
    Csv,
    /// Raw SQL format.
    Raw,
}

impl std::str::FromStr for OutputFormat {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "table" => Ok(Self::Table),
            "json" => Ok(Self::Json),
            "csv" => Ok(Self::Csv),
            "raw" => Ok(Self::Raw),
            _ => Err("Invalid output format"),
        }
    }
}

/// Query context for CLI operations.
#[derive(Debug)]
pub struct QueryContext {
    /// Configuration.
    config: AppConfig,
    /// Selected database profile.
    profile: DatabaseProfile,
    /// Output format.
    output_format: OutputFormat,
    /// Safety level override.
    safety_level: Option<String>,
    /// Disable confirmations.
    no_confirm: bool,
}

impl QueryContext {
    /// Create a new query context.
    ///
    /// # Errors
    /// Returns an error if the configuration cannot be loaded.
    pub fn new(
        config_path: &str,
        profile_name: &str,
        output: &str,
        safety_level: Option<&str>,
        no_confirm: bool,
    ) -> Result<Self> {
        let mut loader = ConfigLoader::new(config_path);
        let config = loader.try_load().with_context(|| {
            format!("Failed to load configuration from {}", config_path)
        })?;

        let profile = config
            .databases
            .iter()
            .find(|p| p.name == profile_name)
            .or(config.databases.first())
            .with_context(|| {
                format!("Database profile '{}' not found", profile_name)
            })?
            .clone();

        let output_format = OutputFormat::from_str(output).unwrap_or(OutputFormat::Table);

        Ok(Self {
            config,
            profile,
            output_format,
            safety_level: safety_level.map(|s| s.to_string()),
            no_confirm,
        })
    }

    /// Get the database URL.
    #[must_use]
    pub fn database_url(&self) -> &str {
        &self.profile.url
    }

    /// Get the output format.
    #[must_use]
    pub fn output_format(&self) -> OutputFormat {
        self.output_format
    }

    /// Get the safety level.
    #[must_use]
    pub fn safety_level(&self) -> Option<&str> {
        self.safety_level.as_deref()
    }

    /// Check if confirmations are disabled.
    #[must_use]
    pub fn no_confirm(&self) -> bool {
        self.no_confirm
    }

    /// Get the config.
    #[must_use]
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
}

/// Result of a query command.
#[derive(Debug)]
pub struct QueryResult {
    /// The generated SQL query.
    pub sql: String,
    /// Query results as JSON.
    pub results: Option<serde_json::Value>,
    /// Number of rows affected.
    pub rows_affected: Option<u64>,
    /// Query execution time in milliseconds.
    pub execution_time_ms: u64,
    /// Whether the query was successful.
    pub success: bool,
    /// Error message if any.
    pub error: Option<String>,
}

impl QueryResult {
    /// Create a successful result.
    #[must_use]
    pub fn success(sql: String, results: Option<serde_json::Value>, rows: Option<u64>, time: u64) -> Self {
        Self {
            sql,
            results,
            rows_affected: rows,
            execution_time_ms: time,
            success: true,
            error: None,
        }
    }

    /// Create a failed result.
    #[must_use]
    pub fn error(sql: String, error: String, time: u64) -> Self {
        Self {
            sql,
            results: None,
            rows_affected: None,
            execution_time_ms: time,
            success: false,
            error: Some(error),
        }
    }
}

/// Format results for output.
pub fn format_results(result: &QueryResult, format: OutputFormat) -> String {
    match format {
        OutputFormat::Json => {
            if let Some(json) = &result.results {
                serde_json::to_string_pretty(json).unwrap_or_default()
            } else if let Some(ref error) = result.error {
                format!("{{\"error\": \"{}\"}}", error)
            } else {
                "{}".to_string()
            }
        }
        OutputFormat::Csv => {
            // Simple CSV output - would need proper implementation
            if let Some(json) = &result.results {
                serde_json::to_string_pretty(json).unwrap_or_default()
            } else {
                "".to_string()
            }
        }
        OutputFormat::Table | OutputFormat::Raw => {
            // For raw output, just show the SQL
            if result.success {
                format!("SQL: {}\nRows: {:?}\nTime: {}ms",
                    result.sql,
                    result.rows_affected,
                    result.execution_time_ms
                )
            } else {
                format!("Error: {}", result.error.as_ref().unwrap_or(&"Unknown error".to_string()))
            }
        }
    }
}

/// Format a message for display.
pub fn format_message(message: &Message) -> String {
    let role = match message.role {
        postgres_agent_core::context::MessageRole::User => "User",
        postgres_agent_core::context::MessageRole::Assistant => "Assistant",
        postgres_agent_core::context::MessageRole::System => "System",
        postgres_agent_core::context::MessageRole::Tool => "Tool",
    };
    format!("[{}]: {}", role, message.content)
}

/// Find configuration file paths.
pub fn find_config_files() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Check current directory
    paths.push(PathBuf::from("config.yaml"));

    // Check home directory
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(home).join(".config").join("pg-agent.yaml"));
    }

    // Check XDG config directory
    if let Ok(config) = std::env::var("XDG_CONFIG_HOME") {
        paths.push(PathBuf::from(config).join("pg-agent.yaml"));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_parsing() {
        assert!(matches!(OutputFormat::from_str("table"), Ok(OutputFormat::Table)));
        assert!(matches!(OutputFormat::from_str("json"), Ok(OutputFormat::Json)));
        assert!(matches!(OutputFormat::from_str("csv"), Ok(OutputFormat::Csv)));
        assert!(matches!(OutputFormat::from_str("invalid"), Err(_)));
    }

    #[test]
    fn test_query_result_success() {
        let result = QueryResult::success(
            "SELECT * FROM users".to_string(),
            Some(serde_json::json!({"users": []})),
            Some(10),
            50,
        );

        assert!(result.success);
        assert_eq!(result.sql, "SELECT * FROM users");
        assert_eq!(result.rows_affected, Some(10));
        assert_eq!(result.execution_time_ms, 50);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_query_result_error() {
        let result = QueryResult::error(
            "SELECT * FROM nonexistent".to_string(),
            "table not found".to_string(),
            10,
        );

        assert!(!result.success);
        assert!(result.results.is_none());
        assert!(result.error.is_some());
        assert_eq!(result.error.as_ref().unwrap(), "table not found");
    }
}
