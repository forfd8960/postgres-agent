//! Core agent logic for PostgreSQL Agent.

#![warn(missing_docs)]

pub mod agent;
pub mod context;
pub mod decision;
pub mod error;

pub use agent::PostgresAgent;
pub use context::AgentContext;
pub use decision::AgentDecision;
pub use error::AgentError;
