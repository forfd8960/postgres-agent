//! Terminal UI for PostgreSQL Agent
//!
//! Interactive terminal-based UI for the PostgreSQL AI Agent.

#![warn(missing_docs)]

pub mod app;
pub mod components;
pub mod views;

pub use app::{AppState, PostgresAgentTui, TuiError, TuiResult, ViewMode};
pub use components::{Command, CommandPalette, Input, InputMode, SafetyLevel, StatusBar, StatusInfo, ConnectionStatus};
pub use views::{ChatMessage, ChatView};
