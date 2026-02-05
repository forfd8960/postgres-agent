//! Secret and sensitive data handling.
//!
//! Provides utilities for securely handling sensitive data like API keys
//! and database passwords using the `secrecy` crate.

use secrecy::{ExposeSecret, Secret};
use std::fmt;

/// A secret string that is not logged or displayed.
///
/// This type wraps sensitive strings and implements `Display` to prevent
/// accidental leakage in logs or error messages.
#[derive(Clone, Debug)]
pub struct SecretString(pub Secret<String>);

impl SecretString {
    /// Create a new secret string from a regular string.
    #[must_use]
    pub fn new(s: impl Into<String>) -> Self {
        Self(Secret::new(s.into()))
    }

    /// Create a secret string from an environment variable.
    ///
    /// # Errors
    /// Returns an error if the environment variable is not set.
    pub fn from_env(var: &str) -> Result<Self, EnvVarError> {
        let value = std::env::var(var).map_err(|_| EnvVarError::NotFound {
            variable: var.to_string(),
        })?;
        Ok(Self::new(value))
    }

    /// Create a secret string from an environment variable with a default.
    #[must_use]
    pub fn from_env_or(var: &str, default: impl Into<String>) -> Self {
        let value = std::env::var(var).unwrap_or_else(|_| default.into());
        Self::new(value)
    }

    /// Get a reference to the inner secret.
    #[must_use]
    pub fn inner(&self) -> &Secret<String> {
        &self.0
    }

    /// Get the secret value as a string.
    ///
    /// # Warning
    /// This exposes the secret value. Use with caution and avoid
    /// logging or displaying the result.
    #[must_use]
    pub fn expose(&self) -> String {
        self.0.expose_secret().clone()
    }

    /// Check if the secret is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.expose_secret().is_empty()
    }

    /// Get the length of the secret.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.expose_secret().len()
    }
}

impl Default for SecretString {
    fn default() -> Self {
        Self(Secret::new(String::new()))
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***REDACTED({} chars)***", self.len())
    }
}

impl From<String> for SecretString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SecretString {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

/// Error type for environment variable operations.
#[derive(Debug, thiserror::Error)]
pub enum EnvVarError {
    #[error("Environment variable not found: {variable}")]
    NotFound { variable: String },
}

/// Result type for secret operations.
pub type SecretResult<T> = Result<T, EnvVarError>;

/// Mask a string for safe logging.
///
/// Returns a masked version of the string where most characters
/// are replaced with asterisks, showing only the first and last few characters.
#[must_use]
pub fn mask_secret(s: &str, visible_chars: usize) -> String {
    if s.len() <= visible_chars * 2 {
        return "***".to_string();
    }

    let prefix = &s[..visible_chars];
    let suffix = &s[s.len().saturating_sub(visible_chars)..];
    let masked_len = s.len().saturating_sub(visible_chars * 2);

    format!("{}***{}{}", prefix, "*".repeat(masked_len.min(20)), suffix)
}

/// Common secret types for the application.
#[derive(Clone, Debug)]
pub struct ApiKey(SecretString);

impl ApiKey {
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self(SecretString::new(key))
    }

    /// Create an API key from an environment variable.
    ///
    /// # Errors
    /// Returns an error if the environment variable is not set.
    pub fn from_env() -> Result<Self, EnvVarError> {
        Ok(Self(SecretString::from_env("OPENAI_API_KEY")?))
    }

    #[must_use]
    pub fn from_env_or_default(default: impl Into<String>) -> Self {
        Self(SecretString::from_env_or("OPENAI_API_KEY", default))
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", mask_secret(self.0.expose().as_str(), 4))
    }
}

impl AsRef<Secret<String>> for ApiKey {
    fn as_ref(&self) -> &Secret<String> {
        self.0.inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_secret() {
        // Short strings are fully masked
        assert_eq!(mask_secret("abc", 2), "***");

        // Longer strings show first and last characters
        assert_eq!(mask_secret("supersecretkey", 3), "sup*******ey");

        // Very long strings are truncated
        let result = mask_secret("a".repeat(100), 3);
        assert!(result.starts_with("a**"));
        assert!(result.ends_with("***a"));
        assert!(result.contains("*******************"));
    }

    #[test]
    fn test_secret_string() {
        let secret = SecretString::new("my-secret-value");
        assert_eq!(secret.len(), 14);
        assert!(!secret.is_empty());
        assert_eq!(format!("{}", secret), "***REDACTED(14 chars)***");
    }

    #[test]
    fn test_api_key_display() {
        let key = ApiKey::new("sk-1234567890abcdef");
        let display = format!("{}", key);
        assert_eq!(display, "sk-1*************ef");
    }
}
