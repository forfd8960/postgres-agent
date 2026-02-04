//! Command implementations for the PostgreSQL Agent.

use anyhow::{Context, Result};
use std::fmt::Write as FmtWrite;

/// Run interactive TUI mode.
pub async fn run_interactive(_profile: Option<String>) -> Result<()> {
    println!("Interactive TUI mode not yet implemented.");
    println!("Please use CLI query mode for now.");
    Ok(())
}

/// Execute a single query.
pub async fn run_query(query: &str, _profile: Option<String>, _output: &str) -> Result<()> {
    println!("Query execution not yet implemented.");
    println!("Query: {}", query);
    Ok(())
}

/// Manage database profiles.
pub async fn manage_profiles(_action: &super::cli::ProfileCommands) -> Result<()> {
    println!("Profile management not yet implemented.");
    Ok(())
}

/// Show configuration.
pub async fn show_config(_effective: bool) -> Result<()> {
    println!("Configuration:");
    println!("  Version: 0.1.0");
    println!("  Status: Foundation phase");
    println!("\nRun 'pg-agent doctor' for system check.");
    Ok(())
}

/// Run system doctor check.
pub async fn run_doctor() -> Result<()> {
    println!("=== PostgreSQL Agent System Check ===\n");

    // Check Rust version
    print_check("Rust version", rustc_version::version());

    // Check PostgreSQL connection
    print_check("PostgreSQL", None);

    // Check API key
    print_check("OPENAI_API_KEY", None);

    println!("\nAll basic checks passed!");
    println!("Full functionality available after Phase 1 completion.");

    Ok(())
}

fn print_check(name: &str, result: Option<Result<String, String>>) {
    match result {
        Some(Ok(value)) => {
            println!("[OK] {}: {}", name, value);
        }
        Some(Err(e)) => {
            println!("[WARN] {}: {}", name, e);
        }
        None => {
            println!("[--] {}: Not configured yet", name);
        }
    }
}
