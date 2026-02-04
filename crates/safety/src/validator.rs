//! Safety validator.

use serde::{Deserialize, Serialize};

use crate::blacklist::{default_blacklist, SqlBlacklist};
use crate::pii::{default_pii_detector, PiiDetector};

/// Safety levels controlling agent behavior.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SafetyLevel {
    /// Maximum safety - read-only, no modifications.
    #[default]
    ReadOnly,
    /// Balanced safety - confirmations for DML/DDL.
    Balanced,
    /// Permissive - faster execution with minimal checks.
    Permissive,
}

/// Safety context for operations.
#[derive(Debug, Clone)]
pub struct SafetyContext {
    /// Current safety level.
    pub level: SafetyLevel,
    /// Whether the current session is read-only.
    pub read_only: bool,
}

impl Default for SafetyContext {
    fn default() -> Self {
        Self {
            level: SafetyLevel::default(),
            read_only: false,
        }
    }
}

/// Result of safety validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    /// Whether the operation is safe.
    pub is_safe: bool,
    /// Warning messages.
    pub warnings: Vec<String>,
    /// Error message if not safe.
    pub error: Option<String>,
}

/// Safety validator for SQL operations.
#[derive(Debug, Default)]
pub struct SafetyValidator {
    /// Blacklisted SQL patterns.
    blacklist: SqlBlacklist,
    /// PII detector.
    pii_detector: PiiDetector,
}

impl SafetyValidator {
    /// Create a new safety validator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            blacklist: default_blacklist(),
            pii_detector: default_pii_detector(),
        }
    }

    /// Validate a SQL query for safety.
    #[allow(dead_code)]
    pub fn validate(&self, sql: &str, ctx: &SafetyContext) -> ValidationResult {
        let mut warnings = Vec::new();

        // Check for blacklisted patterns
        if self.blacklist.contains_blacklisted(sql) {
            return ValidationResult {
                is_safe: false,
                warnings,
                error: Some("Query contains prohibited operation".to_string()),
            };
        }

        // Check for PII
        if self.pii_detector.contains_pii(sql) {
            warnings.push("Query may contain PII".to_string());
        }

        // Check read-only mode
        if ctx.read_only && self.is_mutation(sql) {
            return ValidationResult {
                is_safe: false,
                warnings,
                error: Some("Mutations not allowed in read-only mode".to_string()),
            };
        }

        ValidationResult {
            is_safe: true,
            warnings,
            error: None,
        }
    }

    /// Check if SQL is a mutation (INSERT, UPDATE, DELETE, etc.).
    #[must_use]
    pub fn is_mutation(&self, sql: &str) -> bool {
        let normalized = sql.trim_start().to_uppercase();
        matches!(
            normalized.split_whitespace().next(),
            Some("INSERT" | "UPDATE" | "DELETE" | "ALTER" | "CREATE" | "DROP" | "TRUNCATE")
        )
    }
}
