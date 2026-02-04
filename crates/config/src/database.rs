//! Database profile configuration.

use serde::{Deserialize, Serialize};
use url::Url;

/// Database profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseProfile {
    /// Unique profile name.
    pub name: String,
    /// Connection URL.
    pub url: String,
    /// Optional display name.
    pub display_name: Option<String>,
    /// SSL mode preference.
    #[serde(default = "default_ssl_mode")]
    pub ssl_mode: String,
    /// Connection timeout in seconds.
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
}

fn default_ssl_mode() -> String {
    "prefer".to_string()
}

fn default_connect_timeout() -> u64 {
    30
}

impl DatabaseProfile {
    /// Create a new database profile.
    #[allow(dead_code)]
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            display_name: None,
            ssl_mode: default_ssl_mode(),
            connect_timeout: default_connect_timeout(),
        }
    }

    /// Validate the profile configuration.
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        Url::parse(&self.url)
            .map_err(|_| "Invalid database URL".to_string())?;
        Ok(())
    }
}
