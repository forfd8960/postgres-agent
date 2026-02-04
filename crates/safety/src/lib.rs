//! Safety and audit layer for PostgreSQL Agent.
//!
//! Provides SQL validation, confirmation workflows, audit logging,
//! and PII detection.

#![warn(missing_docs)]

pub mod audit;
pub mod blacklist;
pub mod confirmation;
pub mod pii;
pub mod validator;

pub use audit::{AuditEvent, AuditLogger};
pub use confirmation::{ConfirmationLevel, ConfirmationWorkflow};
pub use validator::{SafetyContext, SafetyLevel, SafetyValidator, ValidationResult};
