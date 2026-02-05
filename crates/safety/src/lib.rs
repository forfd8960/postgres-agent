//! Safety and audit layer for PostgreSQL Agent.
//!
//! This crate provides comprehensive safety features for the PostgreSQL Agent:
//! - SQL validation and classification
//! - Safety levels (ReadOnly, Balanced, Permissive)
//! - Blacklist pattern matching
//! - PII detection and redaction
//! - Confirmation workflows for risky operations
//! - Audit logging for compliance
//!
//! # Example
//!
//! ```rust
//! use postgres_agent_safety::{SafetyValidator, SafetyContext, SafetyLevel};
//!
//! let validator = SafetyValidator::new();
//! let ctx = SafetyContext::with_level(SafetyLevel::Balanced);
//!
//! let result = validator.validate("SELECT * FROM users", &ctx);
//! assert!(result.is_allowed);
//! ```

#![warn(missing_docs)]

pub mod audit;
pub mod blacklist;
pub mod confirmation;
pub mod pii;
pub mod validator;

// Re-export types for convenience
pub use audit::{AuditConfig, AuditEvent, AuditLogger, AuditRecord};
pub use confirmation::{
    ConfirmationLevel, ConfirmationRequest, ConfirmationWorkflow,
};
pub use pii::{PiiDetector, PiiType};
pub use validator::{
    OperationType, SafetyContext, SafetyLevel, SafetyValidator, ValidationDetail,
    ValidationDetailKind, ValidationResult,
};
