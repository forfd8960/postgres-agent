//! Configuration loader.

use std::path::{Path, PathBuf};

use super::{error::ConfigError, AppConfig};

/// Configuration loader with file watching support.
#[derive(Debug)]
pub struct ConfigLoader {
    /// Configuration file path.
    path: PathBuf,
    /// Cached configuration.
    config: Option<AppConfig>,
}

impl ConfigLoader {
    /// Create a new configuration loader.
    #[allow(dead_code)]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            config: None,
        }
    }

    /// Load configuration from file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    #[allow(dead_code)]
    pub fn load(&self) -> Result<AppConfig, ConfigError> {
        if !self.path.exists() {
            return Err(ConfigError::FileNotFound {
                path: self.path.to_string_lossy().to_string(),
            });
        }

        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| ConfigError::Invalid {
                message: format!("Failed to read config file: {}", e),
            })?;

        let config: AppConfig = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError { source: e })?;

        Ok(config)
    }

    /// Try to load configuration, returning default if not found.
    #[allow(dead_code)]
    pub fn try_load(&self) -> AppConfig {
        self.load().unwrap_or_default()
    }

    /// Get the configuration file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new("config.yaml")
    }
}
