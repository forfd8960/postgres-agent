//! PostgreSQL AI Agent
//!
//! An interactive terminal-based agent for querying PostgreSQL databases
//! using natural language, powered by LLMs.

use anyhow::Result;
use clap::{Parser, Subcommand};

/// CLI arguments for the PostgreSQL Agent.
#[derive(Parser, Debug)]
#[command(name = "pg-agent")]
#[command(author = "PostgreSQL Agent Contributors")]
#[command(version = "0.1.0")]
#[command(about = "AI-powered PostgreSQL database assistant", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

/// Available commands for the PostgreSQL Agent.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Execute a single query
    Query {
        /// The natural language query
        query: String,
    },
    /// Check system and configuration
    Doctor,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("PostgreSQL Agent v0.1.0");
    println!("========================\n");

    let args = Args::parse();

    match &args.command {
        Commands::Query { query } => {
            println!("Query: {}", query);
            println!("\nAgent execution coming in Phase 1.");
        }
        Commands::Doctor => {
            println!("=== PostgreSQL Agent System Check ===\n");
            println!("Version: 0.1.0");
            println!("Status: Foundation phase");
            println!("\nFull functionality available after Phase 1 completion.");
        }
    }

    Ok(())
}
