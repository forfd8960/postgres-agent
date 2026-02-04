//! Terminal UI for PostgreSQL Agent
//!
//! Interactive terminal-based UI for the PostgreSQL AI Agent.

/// Available views in the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Chat/conversation view.
    #[default]
    Chat,
    /// Query results view.
    Results,
    /// Schema browser.
    Schema,
}

/// Input component.
#[derive(Debug, Default)]
pub struct Input;

impl Input {
    /// Create new input.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Check if active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        false
    }
}

/// Table component.
#[derive(Debug, Default)]
pub struct Table;

impl Table {
    /// Create new table.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Chat view state.
#[derive(Debug, Default)]
pub struct ChatView;

impl ChatView {
    /// Create new chat view.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Results view state.
#[derive(Debug, Default)]
pub struct ResultsView;

impl ResultsView {
    /// Create new results view.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Schema view state.
#[derive(Debug, Default)]
pub struct SchemaView;

impl SchemaView {
    /// Create new schema view.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Main TUI application state.
#[derive(Debug, Default)]
pub struct PostgresAgentTui {
    /// Current view mode.
    view_mode: ViewMode,
}

impl PostgresAgentTui {
    /// Create a new TUI instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the TUI (stub - full implementation in Phase 2).
    pub async fn run(&mut self) -> Result<(), String> {
        Err("TUI not yet implemented. Use CLI mode.".to_string())
    }
}
