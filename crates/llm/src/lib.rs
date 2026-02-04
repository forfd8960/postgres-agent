//! LLM provider abstraction for PostgreSQL Agent.

#![warn(missing_docs)]

pub mod client;
pub mod conversion;
pub mod error;
pub mod openai;
pub mod provider;
pub mod prompt;

pub use client::LlmClient;
pub use conversion::{to_openai_messages, from_openai_response};
pub use error::LlmError;
pub use openai::OpenAiProvider;
pub use provider::{ProviderConfig, ProviderInfo};
pub use prompt::{PromptBuilder, PromptMessage, PromptRole, SystemPrompt, ConversationHistory};
