//! Safety validator.
//!
//! This module provides the [`SafetyValidator`] for validating SQL operations,
//! classifying operation types, and enforcing safety levels.

use serde::{Deserialize, Serialize};

use crate::blacklist::{default_blacklist, SqlBlacklist};
use crate::pii::{default_pii_detector, PiiDetector};

/// Safety levels controlling agent behavior.
///
/// Each level defines what operations are allowed and what
/// confirmations are required.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum SafetyLevel {
    /// Maximum safety - only read-only SELECT queries allowed.
    #[default]
    ReadOnly,
    /// Balanced safety - allows DML with confirmation, blocks DDL.
    Balanced,
    /// Permissive - allows most operations with minimal checks.
    Permissive,
}

impl SafetyLevel {
    /// Check if DML operations are allowed at this level.
    #[must_use]
    pub fn allows_dml(&self) -> bool {
        matches!(self, SafetyLevel::Balanced | SafetyLevel::Permissive)
    }

    /// Check if DDL operations are allowed at this level.
    #[must_use]
    pub fn allows_ddl(&self) -> bool {
        matches!(self, SafetyLevel::Permissive)
    }

    /// Check if confirmation is required for DML at this level.
    #[must_use]
    pub fn requires_dml_confirmation(&self) -> bool {
        matches!(self, SafetyLevel::ReadOnly | SafetyLevel::Balanced)
    }

    /// Check if confirmation is required for DDL at this level.
    #[must_use]
    pub fn requires_ddl_confirmation(&self) -> bool {
        !matches!(self, SafetyLevel::Permissive)
    }
}

/// Types of SQL operations for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    /// SELECT query (read-only).
    #[default]
    Read,
    /// INSERT statement.
    Insert,
    /// UPDATE statement.
    Update,
    /// DELETE statement.
    Delete,
    /// ALTER TABLE/VIEW/etc.
    Alter,
    /// CREATE TABLE/VIEW/INDEX/etc.
    Create,
    /// DROP statement.
    Drop,
    /// TRUNCATE statement.
    Truncate,
    /// GRANT or REVOKE statement.
    Grant,
    /// VACUUM, ANALYZE, or other maintenance.
    Maintenance,
    /// Transaction control (BEGIN, COMMIT, ROLLBACK).
    Transaction,
    /// Other/unknown operation.
    Other,
}

/// Safety context for operations.
#[derive(Debug, Clone)]
pub struct SafetyContext {
    /// Current safety level.
    pub level: SafetyLevel,
    /// Whether the current session is read-only.
    pub read_only: bool,
    /// User identifier for audit logging.
    pub user_id: Option<String>,
    /// Request ID for tracing.
    pub request_id: Option<String>,
}

impl Default for SafetyContext {
    fn default() -> Self {
        Self {
            level: SafetyLevel::default(),
            read_only: false,
            user_id: None,
            request_id: None,
        }
    }
}

impl SafetyContext {
    /// Create a context with a specific safety level.
    #[must_use]
    pub fn with_level(level: SafetyLevel) -> Self {
        Self {
            level,
            ..Default::default()
        }
    }

    /// Create a read-only context.
    #[must_use]
    pub fn read_only() -> Self {
        Self {
            level: SafetyLevel::ReadOnly,
            read_only: true,
            ..Default::default()
        }
    }

    /// Set the user ID.
    #[must_use]
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the request ID.
    #[must_use]
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// Result of safety validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    /// Whether the operation is allowed.
    pub is_allowed: bool,
    /// Classification of the operation type.
    #[serde(default)]
    pub operation_type: OperationType,
    /// Warning messages.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Error message if not allowed.
    pub error: Option<String>,
    /// Whether the operation requires confirmation.
    pub requires_confirmation: bool,
    /// Details about detected issues.
    #[serde(default)]
    pub details: Vec<ValidationDetail>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_allowed: true,
            operation_type: OperationType::Other,
            warnings: Vec::new(),
            error: None,
            requires_confirmation: false,
            details: Vec::new(),
        }
    }
}

/// Detailed validation information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationDetail {
    /// Type of issue detected.
    pub kind: ValidationDetailKind,
    /// Description of the issue.
    pub message: String,
    /// Position in SQL where issue was found.
    #[serde(default)]
    pub position: Option<usize>,
}

/// Types of validation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationDetailKind {
    /// Blacklisted pattern detected.
    BlacklistMatch,
    /// PII detected.
    PiiDetected,
    /// Mutation in read-only mode.
    MutationInReadOnly,
    /// DDL without confirmation.
    DdlWithoutConfirmation,
    /// Large operation detected.
    LargeOperation,
    /// Potential SQL injection.
    PotentialInjection,
}

/// Safety validator for SQL operations.
#[derive(Debug)]
pub struct SafetyValidator {
    /// Blacklisted SQL patterns.
    blacklist: SqlBlacklist,
    /// PII detector.
    pii_detector: PiiDetector,
    /// Maximum rows for a safe query (0 = unlimited).
    max_rows: usize,
    /// Whether to allow maintenance operations.
    allow_maintenance: bool,
}

impl Default for SafetyValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl SafetyValidator {
    /// Create a new safety validator with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            blacklist: default_blacklist(),
            pii_detector: default_pii_detector(),
            max_rows: 0,
            allow_maintenance: false,
        }
    }

    /// Create a validator with custom max rows.
    #[must_use]
    pub fn with_max_rows(mut self, max_rows: usize) -> Self {
        self.max_rows = max_rows;
        self
    }

    /// Create a validator that allows maintenance operations.
    #[must_use]
    pub fn with_maintenance_allowed(mut self) -> Self {
        self.allow_maintenance = true;
        self
    }

    /// Validate a SQL query for safety.
    pub fn validate(&self, sql: &str, ctx: &SafetyContext) -> ValidationResult {
        let mut result = ValidationResult::default();

        // Classify the operation type
        result.operation_type = self.classify_operation(sql);

        // Check for blacklisted patterns
        if let Some(match_info) = self.blacklist.find_match(sql) {
            result.is_allowed = false;
            result.error = Some(format!("Query contains prohibited operation: {}", match_info));
            result.details.push(ValidationDetail {
                kind: ValidationDetailKind::BlacklistMatch,
                message: format!("Blacklisted pattern matched: {}", match_info),
                position: None,
            });
            return result;
        }

        // Check for PII
        if self.pii_detector.contains_pii(sql) {
            result.warnings.push("Query may contain PII".to_string());
            result.details.push(ValidationDetail {
                kind: ValidationDetailKind::PiiDetected,
                message: "Potential PII detected in query".to_string(),
                position: None,
            });
        }

        // Check read-only mode
        if ctx.read_only && result.operation_type != OperationType::Read {
            result.is_allowed = false;
            result.error = Some("Mutations not allowed in read-only mode".to_string());
            result.details.push(ValidationDetail {
                kind: ValidationDetailKind::MutationInReadOnly,
                message: "Query is not read-only but context is read-only".to_string(),
                position: None,
            });
            return result;
        }

        // Check safety level for operation type
        match result.operation_type {
            OperationType::Read => {
                // Always allowed
            }
            OperationType::Insert | OperationType::Update | OperationType::Delete => {
                if !ctx.level.allows_dml() {
                    result.is_allowed = false;
                    result.error = Some(format!(
                        "DML operations ({}) not allowed at {:?} safety level",
                        result.operation_type.label(),
                        ctx.level
                    ));
                    return result;
                }
                if ctx.level.requires_dml_confirmation() {
                    result.requires_confirmation = true;
                }
            }
            OperationType::Alter | OperationType::Create | OperationType::Drop | OperationType::Truncate => {
                if !ctx.level.allows_ddl() {
                    result.is_allowed = false;
                    result.error = Some(format!(
                        "DDL operations ({}) not allowed at {:?} safety level",
                        result.operation_type.label(),
                        ctx.level
                    ));
                    return result;
                }
                if ctx.level.requires_ddl_confirmation() {
                    result.requires_confirmation = true;
                }
            }
            OperationType::Grant => {
                result.is_allowed = false;
                result.error = Some("GRANT/REVOKE operations are not allowed".to_string());
                return result;
            }
            OperationType::Maintenance => {
                if !self.allow_maintenance {
                    result.is_allowed = false;
                    result.error = Some("Maintenance operations not allowed".to_string());
                    return result;
                }
            }
            OperationType::Transaction | OperationType::Other => {
                // Allow by default, may want to add more checks
            }
        }

        result
    }

    /// Classify a SQL operation into its type.
    #[must_use]
    pub fn classify_operation(&self, sql: &str) -> OperationType {
        let normalized = sql.trim_start().to_uppercase();

        if normalized.starts_with("SELECT") {
            OperationType::Read
        } else if normalized.starts_with("INSERT") {
            OperationType::Insert
        } else if normalized.starts_with("UPDATE") {
            OperationType::Update
        } else if normalized.starts_with("DELETE") {
            OperationType::Delete
        } else if normalized.starts_with("ALTER") {
            OperationType::Alter
        } else if normalized.starts_with("CREATE") {
            OperationType::Create
        } else if normalized.starts_with("DROP") {
            OperationType::Drop
        } else if normalized.starts_with("TRUNCATE") {
            OperationType::Truncate
        } else if normalized.starts_with("GRANT") || normalized.starts_with("REVOKE") {
            OperationType::Grant
        } else if normalized.starts_with("VACUUM") || normalized.starts_with("ANALYZE")
            || normalized.starts_with("REINDEX")
        {
            OperationType::Maintenance
        } else if normalized.starts_with("BEGIN")
            || normalized.starts_with("COMMIT")
            || normalized.starts_with("ROLLBACK")
            || normalized.starts_with("SAVEPOINT")
        {
            OperationType::Transaction
        } else {
            OperationType::Other
        }
    }

    /// Check if SQL is a mutation (non-SELECT).
    #[must_use]
    pub fn is_mutation(&self, sql: &str) -> bool {
        self.classify_operation(sql) != OperationType::Read
    }

    /// Check if SQL is a DDL operation.
    #[must_use]
    pub fn is_ddl(&self, sql: &str) -> bool {
        matches!(
            self.classify_operation(sql),
            OperationType::Alter | OperationType::Create | OperationType::Drop | OperationType::Truncate
        )
    }

    /// Check if SQL is a DML operation.
    #[must_use]
    pub fn is_dml(&self, sql: &str) -> bool {
        matches!(
            self.classify_operation(sql),
            OperationType::Insert | OperationType::Update | OperationType::Delete
        )
    }

    /// Get the PII detector for redaction.
    #[must_use]
    pub fn pii_detector(&self) -> &PiiDetector {
        &self.pii_detector
    }
}

impl OperationType {
    /// Get a human-readable label for the operation type.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Read => "SELECT",
            Self::Insert => "INSERT",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Alter => "ALTER",
            Self::Create => "CREATE",
            Self::Drop => "DROP",
            Self::Truncate => "TRUNCATE",
            Self::Grant => "GRANT/REVOKE",
            Self::Maintenance => "MAINTENANCE",
            Self::Transaction => "TRANSACTION",
            Self::Other => "OTHER",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_level_allows() {
        assert!(!SafetyLevel::ReadOnly.allows_dml());
        assert!(SafetyLevel::Balanced.allows_dml());
        assert!(SafetyLevel::Permissive.allows_dml());

        assert!(!SafetyLevel::ReadOnly.allows_ddl());
        assert!(!SafetyLevel::Balanced.allows_ddl());
        assert!(SafetyLevel::Permissive.allows_ddl());
    }

    #[test]
    fn test_operation_classification() {
        let validator = SafetyValidator::new();

        assert_eq!(validator.classify_operation("SELECT * FROM users"), OperationType::Read);
        assert_eq!(validator.classify_operation("  select 1"), OperationType::Read);
        assert_eq!(validator.classify_operation("INSERT INTO users VALUES (1)"), OperationType::Insert);
        assert_eq!(validator.classify_operation("UPDATE users SET name = 'test'"), OperationType::Update);
        assert_eq!(validator.classify_operation("DELETE FROM users"), OperationType::Delete);
        assert_eq!(validator.classify_operation("ALTER TABLE users ADD COLUMN age INT"), OperationType::Alter);
        assert_eq!(validator.classify_operation("CREATE TABLE new_table (id INT)"), OperationType::Create);
        assert_eq!(validator.classify_operation("DROP TABLE users"), OperationType::Drop);
        assert_eq!(validator.classify_operation("TRUNCATE TABLE users"), OperationType::Truncate);
        assert_eq!(validator.classify_operation("GRANT SELECT ON users TO app"), OperationType::Grant);
        assert_eq!(validator.classify_operation("VACUUM ANALYZE"), OperationType::Maintenance);
        assert_eq!(validator.classify_operation("BEGIN"), OperationType::Transaction);
    }

    #[test]
    fn test_validation_read_only() {
        let validator = SafetyValidator::new();
        let ctx = SafetyContext::read_only();

        let result = validator.validate("SELECT 1", &ctx);
        assert!(result.is_allowed);
        assert_eq!(result.operation_type, OperationType::Read);

        let result = validator.validate("INSERT INTO users VALUES (1)", &ctx);
        assert!(!result.is_allowed);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_validation_blacklist() {
        let validator = SafetyValidator::new();
        let ctx = SafetyContext::default();

        let result = validator.validate("DROP TABLE users", &ctx);
        assert!(!result.is_allowed);
        assert_eq!(result.error, Some("Query contains prohibited operation: DROP".to_string()));
    }
}
