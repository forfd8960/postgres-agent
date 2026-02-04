//! LLM provider errors.

use thiserror::Error;

/// Errors from LLM operations.
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("API error: {message}")]
    ApiError { message: String },

    #[error("No response received")]
    NoResponse,

    #[error("Rate limited: retry after {retry_after}s")]
    RateLimited { retry_after: u64 },
}
