//! Confirmation workflow.
//!
//! This module provides the [`ConfirmationWorkflow`] for managing
//! user confirmations for potentially risky operations.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use uuid::Uuid;

use crate::validator::{OperationType, SafetyLevel};

/// Confirmation level for operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
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

impl ConfirmationLevel {
    /// Check if this level requires user input.
    #[must_use]
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Simple | Self::Typed | Self::AdminApproval)
    }

    /// Get the confirmation prompt message.
    #[must_use]
    pub fn prompt_message(&self, operation: &str) -> String {
        match self {
            Self::None => String::new(),
            Self::Simple => format!("Are you sure you want to {}? (y/n)", operation),
            Self::Typed => format!(
                "Type the operation name to confirm: Type '{}' to proceed",
                operation.to_uppercase()
            ),
            Self::AdminApproval => {
                format!("This operation requires admin approval: {}", operation)
            }
        }
    }
}

/// Confirmation request state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationRequest {
    /// Unique request ID.
    pub id: String,
    /// Operation description.
    pub operation: String,
    /// SQL being confirmed.
    pub sql: String,
    /// Confirmation level required.
    pub level: ConfirmationLevel,
    /// When the request was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Whether this request has expired.
    pub expired: bool,
}

impl Default for ConfirmationRequest {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation: String::new(),
            sql: String::new(),
            level: ConfirmationLevel::None,
            created_at: chrono::Utc::now(),
            expired: false,
        }
    }
}

impl ConfirmationRequest {
    /// Create a new confirmation request.
    #[must_use]
    pub fn new(operation: String, sql: String, level: ConfirmationLevel) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation,
            sql,
            level,
            created_at: chrono::Utc::now(),
            expired: false,
        }
    }

    /// Check if the request has expired (default 5 minutes).
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let expiry = chrono::Duration::minutes(5);
        chrono::Utc::now() - self.created_at > expiry
    }
}

/// Confirmation workflow state.
#[derive(Debug)]
pub struct ConfirmationWorkflow {
    /// Pending confirmation request.
    pending: Option<ConfirmationRequest>,
    /// Confirmation response (for testing/automation).
    auto_confirm: Arc<AtomicBool>,
    /// Typed confirmation value (for "Typed" level).
    typed_confirmation: Arc<AtomicBool>,
    /// Expected typed value.
    expected_typed_value: String,
}

impl Default for ConfirmationWorkflow {
    fn default() -> Self {
        Self {
            pending: None,
            auto_confirm: Arc::new(AtomicBool::new(false)),
            typed_confirmation: Arc::new(AtomicBool::new(false)),
            expected_typed_value: String::new(),
        }
    }
}

impl ConfirmationWorkflow {
    /// Create a new confirmation workflow.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a workflow that auto-confirms (for testing).
    #[must_use]
    pub fn with_auto_confirm() -> Self {
        let mut workflow = Self::default();
        workflow.auto_confirm = Arc::new(AtomicBool::new(true));
        workflow
    }

    /// Request confirmation for an operation.
    pub fn request(
        &mut self,
        operation: &str,
        sql: &str,
        level: ConfirmationLevel,
    ) -> Option<ConfirmationRequest> {
        if !level.requires_confirmation() {
            return None;
        }

        let request = ConfirmationRequest::new(operation.to_string(), sql.to_string(), level);
        self.pending = Some(request.clone());
        self.expected_typed_value = operation.to_uppercase();

        Some(request)
    }

    /// Check if confirmation is pending.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.pending.is_some() && !self.pending.as_ref().unwrap().expired
    }

    /// Get the pending confirmation request.
    #[must_use]
    pub fn pending_request(&self) -> Option<&ConfirmationRequest> {
        self.pending.as_ref()
    }

    /// Get the confirmation prompt.
    #[must_use]
    pub fn get_prompt(&self) -> Option<String> {
        self.pending.as_ref().map(|r| r.level.prompt_message(&r.operation))
    }

    /// Confirm the operation (simple confirmation).
    pub fn confirm(&mut self) -> bool {
        if self.auto_confirm.load(Ordering::SeqCst) {
            self.clear();
            return true;
        }

        if let Some(ref mut request) = self.pending {
            if request.level == ConfirmationLevel::Simple
                || request.level == ConfirmationLevel::Typed
            {
                request.expired = true;
                self.clear();
                return true;
            }
        }
        false
    }

    /// Confirm with typed value (for Typed confirmation level).
    pub fn confirm_typed(&mut self, value: &str) -> bool {
        if self.auto_confirm.load(Ordering::SeqCst) {
            self.clear();
            return true;
        }

        if let Some(ref request) = self.pending {
            if request.level == ConfirmationLevel::Typed && value.trim() == self.expected_typed_value {
                self.clear();
                return true;
            }
        }
        false
    }

    /// Approve as admin.
    pub fn admin_approve(&mut self) -> bool {
        if self.auto_confirm.load(Ordering::SeqCst) {
            self.clear();
            return true;
        }

        if let Some(ref mut request) = self.pending {
            if request.level == ConfirmationLevel::AdminApproval {
                request.expired = true;
                self.clear();
                return true;
            }
        }
        false
    }

    /// Cancel the pending confirmation.
    pub fn cancel(&mut self) {
        if let Some(ref mut request) = self.pending {
            request.expired = true;
        }
        self.clear();
    }

    /// Clear the pending confirmation.
    fn clear(&mut self) {
        self.pending = None;
        self.expected_typed_value.clear();
    }

    /// Check if the current level requires typed confirmation.
    #[must_use]
    pub fn requires_typed_input(&self) -> bool {
        self.pending
            .as_ref()
            .map(|r| r.level == ConfirmationLevel::Typed)
            .unwrap_or(false)
    }

    /// Get the expected typed value (for display purposes).
    #[must_use]
    pub fn expected_type_value(&self) -> &str {
        &self.expected_typed_value
    }
}

/// Convenience function to check if a validation result requires confirmation.
#[must_use]
pub fn requires_confirmation(
    result: &crate::validator::ValidationResult,
    level: SafetyLevel,
) -> bool {
    if !result.is_allowed {
        return false;
    }

    match result.operation_type {
        OperationType::Read => false,
        OperationType::Insert | OperationType::Update | OperationType::Delete => {
            level.requires_dml_confirmation()
        }
        OperationType::Alter
        | OperationType::Create
        | OperationType::Drop
        | OperationType::Truncate => level.requires_ddl_confirmation(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirmation_levels() {
        assert!(!ConfirmationLevel::None.requires_confirmation());
        assert!(ConfirmationLevel::Simple.requires_confirmation());
        assert!(ConfirmationLevel::Typed.requires_confirmation());
        assert!(ConfirmationLevel::AdminApproval.requires_confirmation());
    }

    #[test]
    fn test_request_creation() {
        let request = ConfirmationRequest::new(
            "DELETE".to_string(),
            "DELETE FROM users".to_string(),
            ConfirmationLevel::Typed,
        );

        assert_eq!(request.operation, "DELETE");
        assert_eq!(request.sql, "DELETE FROM users");
        assert_eq!(request.level, ConfirmationLevel::Typed);
        assert!(!request.is_expired());
    }

    #[test]
    fn test_workflow_request() {
        let mut workflow = ConfirmationWorkflow::new();

        let request = workflow.request("DELETE", "DELETE FROM users", ConfirmationLevel::Simple);

        assert!(request.is_some());
        assert!(workflow.is_pending());
        assert!(workflow.get_prompt().is_some());
    }

    #[test]
    fn test_workflow_auto_confirm() {
        let mut workflow = ConfirmationWorkflow::with_auto_confirm();

        let request = workflow.request("DELETE", "DELETE FROM users", ConfirmationLevel::Typed);

        assert!(request.is_some());
        assert!(workflow.confirm());
        assert!(!workflow.is_pending());
    }

    #[test]
    fn test_workflow_cancel() {
        let mut workflow = ConfirmationWorkflow::new();

        workflow.request("DELETE", "DELETE FROM users", ConfirmationLevel::Simple);
        assert!(workflow.is_pending());

        workflow.cancel();
        assert!(!workflow.is_pending());
    }
}
