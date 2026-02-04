//! PostgreSQL Agent
//!
//! An interactive terminal-based agent for querying PostgreSQL databases
//! using natural language, powered by LLMs.
//!
//! # Example
//!
//! ```rust
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     println!("PostgreSQL Agent");
//!     Ok(())
//! }
//! ```

/// Re-export commonly used types from submodules.
pub use postgres_agent_core as core;
pub use postgres_agent_llm as llm;
pub use postgres_agent_db as db;
pub use postgres_agent_tools as tools;
// pub use postgres_agent_tui as tui;
pub use postgres_agent_config as config;
pub use postgres_agent_safety as safety;
pub use postgres_agent_cli as cli;
pub use postgres_agent_util as util;
