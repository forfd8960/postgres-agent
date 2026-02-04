//! Confirmation workflow.

use serde::{Deserialize, Serialize};

/// Confirmation level for operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConfirmationLevel {
    /// No confirmation needed.
    #[default]
    None,
    /// Simple confirmation (y/n).
    Simple,
    /// Type-specific confirmation (must type "DELETE" to delete).
    Typed,
    /// Requires explicit approval from admin.
    AdminApproval,
}

/// Confirmation workflow state.
#[derive(Debug, Default)]
pub struct ConfirmationWorkflow {
    /// Current confirmation level.
    level: ConfirmationLevel,
    /// Pending confirmation.
    pending: bool,
    /// Operation description.
    operation: String,
}

impl ConfirmationWorkflow {
    /// Create a new confirmation workflow.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Request confirmation for an operation.
    #[allow(dead_code)]
    pub fn request(&mut self, operation: &str, level: ConfirmationLevel) {
        self.operation = operation.to_string();
        self.level = level;
        self.pending = true;
    }

    /// Check if confirmation is pending.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.pending
    }

    /// Get the current level.
    #[must_use]
    pub fn level(&self) -> ConfirmationLevel {
        self.level
    }

    /// Confirm the operation.
    #[allow(dead_code)]
    pub fn confirm(&mut self) {
        self.pending = false;
    }

    /// Cancel the operation.
    #[allow(dead_code)]
    pub fn cancel(&mut self) {
        self.pending = false;
        self.operation.clear();
    }
}
