//! Command-line argument parsing.
//!
//! This module provides clap-based argument parsing for the PostgreSQL Agent CLI.

use clap::{Parser, Subcommand};

/// PostgreSQL AI Agent - Query databases using natural language
#[derive(Parser, Debug)]
#[command(name = "pg-agent")]
#[command(author = "PostgreSQL Agent Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Query PostgreSQL databases using natural language", long_about = None)]
pub struct CliArgs {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    pub config: String,

    /// Log level (debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    /// Database profile to use
    #[arg(short, long, default_value = "default")]
    pub profile: String,

    /// Safety level (read_only, balanced, permissive)
    #[arg(short, long)]
    pub safety_level: Option<String>,

    /// Disable confirmation prompts
    #[arg(long, default_value = "false")]
    pub no_confirm: bool,

    /// Output format (json, table, csv)
    #[arg(long, default_value = "table")]
    pub output: String,

    /// Quiet mode (only show results)
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,

    /// Disable TUI and use CLI mode
    #[arg(long, default_value = "false")]
    pub no_tui: bool,

    /// Subcommand to run
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// CLI commands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query the database with natural language
    #[command(arg_required_else_help = true)]
    Query {
        /// Natural language query
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,
    },

    /// Start interactive REPL mode
    #[command(name = "interactive")]
    Interactive {
        /// Database profile to use
        #[arg(short, long, default_value = "default")]
        profile: String,
    },

    /// Run a SQL file
    #[command(name = "exec")]
    Execute {
        /// SQL file to execute
        #[arg(trailing_var_arg = true)]
        files: Vec<String>,
    },

    /// List available database profiles
    #[command(name = "profiles")]
    Profiles,

    /// Show current configuration
    #[command(name = "config")]
    Config,

    /// Show schema information
    #[command(name = "schema")]
    Schema {
        /// Table name filter
        #[arg(short, long)]
        table: Option<String>,
    },

    /// Run system health checks
    #[command(name = "doctor")]
    Doctor,

    /// Show version and exit
    #[command(name = "version")]
    Version,
}

impl CliArgs {
    /// Get the query string from arguments.
    #[must_use]
    pub fn get_query(&self) -> Option<String> {
        match &self.command {
            Some(Commands::Query { query }) if !query.is_empty() => {
                Some(query.join(" "))
            }
            _ => None,
        }
    }

    /// Get the files from arguments.
    #[must_use]
    pub fn get_files(&self) -> Option<Vec<String>> {
        match &self.command {
            Some(Commands::Execute { files }) if !files.is_empty() => {
                Some(files.clone())
            }
            _ => None,
        }
    }

    /// Check if running in interactive mode.
    #[must_use]
    pub fn is_interactive(&self) -> bool {
        matches!(
            self.command,
            Some(Commands::Interactive { .. })
        ) || self.no_tui
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_command_parsing() {
        let args = CliArgs::parse_from([
            "pg-agent",
            "--config", "test.yaml",
            "query",
            "Show me all users",
        ]);

        assert_eq!(args.config, "test.yaml");
        assert_eq!(
            args.get_query(),
            Some("Show me all users".to_string())
        );
    }

    #[test]
    fn test_interactive_command() {
        let args = CliArgs::parse_from([
            "pg-agent",
            "interactive",
            "--profile", "production",
        ]);

        assert!(args.is_interactive());
        match &args.command {
            Some(Commands::Interactive { profile }) => {
                assert_eq!(profile, "production");
            }
            _ => panic!("Expected Interactive command"),
        }
    }

    #[test]
    fn test_version_command() {
        let args = CliArgs::parse_from(["pg-agent", "version"]);
        assert!(matches!(args.command, Some(Commands::Version)));
    }

    #[test]
    fn test_profiles_command() {
        let args = CliArgs::parse_from(["pg-agent", "profiles"]);
        assert!(matches!(args.command, Some(Commands::Profiles)));
    }

    #[test]
    fn test_schema_command() {
        let args = CliArgs::parse_from(["pg-agent", "schema", "--table", "users"]);
        match &args.command {
            Some(Commands::Schema { table }) => {
                assert_eq!(table.as_ref().map(|s| s.as_str()), Some("users"));
            }
            _ => panic!("Expected Schema command"),
        }
    }

    #[test]
    fn test_default_values() {
        let args = CliArgs::parse_from(["pg-agent"]);

        assert_eq!(args.config, "config.toml");
        assert_eq!(args.log_level, "info");
        assert_eq!(args.profile, "default");
        assert!(!args.no_confirm);
        assert!(!args.is_interactive());
    }
}
