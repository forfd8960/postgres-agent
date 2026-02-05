//! PostgreSQL AI Agent - Main entry point
//!
//! An interactive terminal-based agent for querying PostgreSQL databases
//! using natural language, powered by LLMs.

mod commands;

use anyhow::Result;
use clap::Parser;
use postgres_agent_cli::CliArgs;
use tracing_subscriber::EnvFilter;

/// Configure logging based on log level.
fn configure_logging(log_level: &str) {
    let env_filter = EnvFilter::new(log_level);
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = CliArgs::parse();

    // Configure logging
    configure_logging(&args.log_level);

    // Display version info if quiet mode is off
    if !args.quiet {
        println!("PostgreSQL Agent v0.1.0");
        println!("{}\n", "=".repeat(50));
    }

    // Handle commands
    match &args.command {
        Some(postgres_agent_cli::Commands::Query { query }) => {
            let query_str = query.join(" ");
            commands::run_query(
                &query_str,
                &args.config,
                &args.profile,
                &args.output.to_string(),
                args.safety_level.as_deref(),
                args.no_confirm,
                args.quiet,
            )
            .await?;
        }
        Some(postgres_agent_cli::Commands::Interactive { profile }) => {
            commands::run_interactive(
                &args.config,
                profile,
                args.safety_level.as_deref(),
                args.no_confirm,
            )
            .await?;
        }
        Some(postgres_agent_cli::Commands::Execute { files }) => {
            commands::execute_files(
                files,
                &args.config,
                &args.profile,
                &args.output.to_string(),
                args.quiet,
            )
            .await?;
        }
        Some(postgres_agent_cli::Commands::Profiles) => {
            commands::list_profiles(&args.config).await?;
        }
        Some(postgres_agent_cli::Commands::Config) => {
            commands::show_config(&args.config, false).await?;
        }
        Some(postgres_agent_cli::Commands::Schema { table }) => {
            commands::show_schema(&args.config, &args.profile, table.as_deref()).await?;
        }
        Some(postgres_agent_cli::Commands::Doctor) => {
            commands::run_doctor(&args.config).await?;
        }
        Some(postgres_agent_cli::Commands::Version) => {
            println!("PostgreSQL Agent v0.1.0");
        }
        None => {
            // Show help
            println!("PostgreSQL Agent v0.1.0");
            println!();
            println!("Usage: pg-agent [OPTIONS] [COMMAND]");
            println!();
            println!("Commands:");
            println!("  query <text>      Query the database with natural language");
            println!("  interactive       Start interactive REPL mode");
            println!("  exec <files>      Execute SQL files");
            println!("  profiles         List available database profiles");
            println!("  config           Show current configuration");
            println!("  schema           Show database schema");
            println!("  doctor          Run system health checks");
            println!("  version         Show version information");
            println!();
            println!("Run 'pg-agent --help' for more information.");
        }
    }

    Ok(())
}
