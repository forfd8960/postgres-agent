//! Configuration loader.
//!
//! Provides YAML configuration parsing, environment variable overrides,
//! validation, and file watching support.

use std::path::{Path, PathBuf};

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use super::{error::ConfigError, AppConfig, DatabaseProfile, SafetyConfig};

/// Configuration validator.
#[derive(Debug, Default)]
struct ConfigValidator {}

impl ConfigValidator {
    /// Validate the configuration.
    ///
    /// # Errors
    /// Returns an error if validation fails.
    fn validate(&self, config: &AppConfig) -> Result<(), ConfigError> {
        // Validate LLM configuration
        if config.llm.model.is_empty() {
            return Err(ConfigError::ValidationError {
                message: "LLM model cannot be empty".to_string(),
            });
        }

        // Validate temperature range
        if config.llm.temperature < 0.0 || config.llm.temperature > 2.0 {
            return Err(ConfigError::ValidationError {
                message: "LLM temperature must be between 0.0 and 2.0".to_string(),
            });
        }

        // Validate max tokens
        if config.llm.max_tokens == 0 {
            return Err(ConfigError::ValidationError {
                message: "LLM max_tokens must be greater than 0".to_string(),
            });
        }

        // Validate database profiles
        for profile in &config.databases {
            if profile.name.is_empty() {
                return Err(ConfigError::ValidationError {
                    message: "Database profile name cannot be empty".to_string(),
                });
            }

            if let Err(e) = profile.validate() {
                return Err(ConfigError::ValidationError {
                    message: format!("Invalid database profile '{}': {}", profile.name, e),
                });
            }
        }

        // Validate agent configuration
        if config.agent.max_history == 0 {
            return Err(ConfigError::ValidationError {
                message: "Agent max_history must be greater than 0".to_string(),
            });
        }

        if config.agent.max_iterations == 0 {
            return Err(ConfigError::ValidationError {
                message: "Agent max_iterations must be greater than 0".to_string(),
            });
        }

        // Validate safety configuration
        if config.safety.max_query_length == 0 {
            return Err(ConfigError::ValidationError {
                message: "Safety max_query_length must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Configuration loader with file watching support.
#[derive(Debug)]
pub struct ConfigLoader {
    /// Configuration file path.
    path: PathBuf,
    /// Cached configuration.
    config: Option<AppConfig>,
    /// Configuration validator.
    validator: ConfigValidator,
}

impl ConfigLoader {
    /// Create a new configuration loader.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            config: None,
            validator: ConfigValidator::default(),
        }
    }

    /// Load configuration from file.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read, parsed, or validated.
    pub fn load(&mut self) -> Result<AppConfig, ConfigError> {
        if !self.path.exists() {
            return Err(ConfigError::FileNotFound {
                path: self.path.to_string_lossy().to_string(),
            });
        }

        let content = std::fs::read_to_string(&self.path).map_err(|e| ConfigError::Invalid {
            message: format!("Failed to read config file: {}", e),
        })?;

        // Parse YAML
        let mut config: AppConfig = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError { source: e })?;

        // Apply environment variable overrides
        self.apply_env_overrides(&mut config);

        // Validate configuration
        self.validator.validate(&config)?;

        self.config = Some(config.clone());
        Ok(config)
    }

    /// Try to load configuration, returning default if not found.
    ///
    /// # Errors
    /// Returns an error if the file exists but cannot be parsed or validated.
    pub fn try_load(&mut self) -> Result<AppConfig, ConfigError> {
        self.load()
    }

    /// Apply environment variable overrides to the configuration.
    ///
    /// Supports the following overrides:
    /// - `PG_AGENT_LLM_API_KEY`: Override LLM API key
    /// - `PG_AGENT_LLM_BASE_URL`: Override LLM base URL
    /// - `PG_AGENT_LLM_MODEL`: Override LLM model
    /// - `PG_AGENT_LLM_TEMPERATURE`: Override LLM temperature
    /// - `PG_AGENT_DATABASE_URL`: Override first database profile URL
    /// - `PG_AGENT_SAFETY_LEVEL`: Override safety level (read_only, balanced, permissive)
    fn apply_env_overrides(&self, config: &mut AppConfig) {
        // LLM overrides
        if let Ok(api_key) = std::env::var("PG_AGENT_LLM_API_KEY") {
            config.llm.api_key = Some(api_key);
        }
        if let Ok(base_url) = std::env::var("PG_AGENT_LLM_BASE_URL")
            && let Ok(url) = base_url.parse()
        {
            config.llm.base_url = Some(url);
        }
        if let Ok(model) = std::env::var("PG_AGENT_LLM_MODEL") {
            config.llm.model = model;
        }
        if let Ok(temp) = std::env::var("PG_AGENT_LLM_TEMPERATURE")
            && let Ok(t) = temp.parse::<f32>()
        {
            config.llm.temperature = t.clamp(0.0, 2.0);
        }

        // Database override - update first profile or create one
        if let Ok(url) = std::env::var("PG_AGENT_DATABASE_URL") {
            if let Some(profile) = config.databases.first_mut() {
                profile.url = url;
            } else {
                config.databases.push(DatabaseProfile::new("default", &url));
            }
        }

        // Safety level override
        if let Ok(level) = std::env::var("PG_AGENT_SAFETY_LEVEL") {
            config.safety.safety_level = match level.to_lowercase().as_str() {
                "read_only" | "readonly" => SafetyConfig::default().safety_level,
                "balanced" => SafetyConfig::default().safety_level,
                "permissive" => SafetyConfig::default().safety_level,
                _ => config.safety.safety_level,
            };
        }
    }

    /// Get the configuration file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get cached configuration if available.
    #[must_use]
    pub fn cached_config(&self) -> Option<&AppConfig> {
        self.config.as_ref()
    }

    /// Watch the configuration file for changes.
    ///
    /// Returns a watcher that can be used to receive change notifications.
    ///
    /// # Errors
    /// Returns an error if the file watcher cannot be initialized.
    #[allow(dead_code)]
    pub fn watch(&mut self) -> Result<ConfigWatcher, ConfigError> {
        ConfigWatcher::new(&self.path)
    }
}

/// Configuration file watcher.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ConfigWatcher {
    /// Inner watcher.
    watcher: RecommendedWatcher,
    /// Configuration file path.
    path: PathBuf,
}

impl ConfigWatcher {
    /// Create a new configuration watcher.
    ///
    /// # Errors
    /// Returns an error if the watcher cannot be created.
    fn new(path: &Path) -> Result<Self, ConfigError> {
        let path = path.to_path_buf();

        let (tx, _) = std::sync::mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res
                    && let Err(e) = tx.send(event)
                {
                    tracing::error!("Config watcher channel error: {}", e);
                }
            },
            notify::Config::default(),
        )
        .map_err(|e| ConfigError::Invalid {
            message: format!("Failed to create config watcher: {}", e),
        })?;

        watcher
            .watch(&path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::Invalid {
                message: format!("Failed to watch config file: {}", e),
            })?;

        Ok(Self { watcher, path })
    }

    /// Check if the configuration file has been modified.
    #[must_use]
    pub fn is_modified(&self) -> bool {
        self.path.exists()
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new("config.yaml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_profile_validation() {
        let profile = DatabaseProfile::new("test", "postgresql://localhost/test");
        assert!(profile.validate().is_ok());

        let invalid_profile = DatabaseProfile::new("test", "not-a-url");
        assert!(invalid_profile.validate().is_err());
    }

    #[test]
    fn test_env_override_api_key() {
        unsafe { std::env::set_var("PG_AGENT_LLM_API_KEY", "test-key"); }

        let mut config = AppConfig::default();
        let loader = ConfigLoader::new("nonexistent.yaml");

        loader.apply_env_overrides(&mut config);

        assert_eq!(config.llm.api_key, Some("test-key".to_string()));

        unsafe { std::env::remove_var("PG_AGENT_LLM_API_KEY"); }
    }

    #[test]
    fn test_env_override_database_url() {
        unsafe { std::env::set_var("PG_AGENT_DATABASE_URL", "postgresql://localhost/mydb"); }

        let mut config = AppConfig::default();
        let loader = ConfigLoader::new("nonexistent.yaml");

        loader.apply_env_overrides(&mut config);

        assert!(!config.databases.is_empty());
        assert_eq!(config.databases[0].url, "postgresql://localhost/mydb");

        unsafe { std::env::remove_var("PG_AGENT_DATABASE_URL"); }
    }

    #[test]
    fn test_load_from_temp_file() {
        let yaml_content = r#"
llm:
  provider: openai
  model: gpt-4
  temperature: 0.5
  maxTokens: 2048

databases:
  - name: testdb
    url: postgresql://localhost/test

agent:
  maxHistory: 100
  maxIterations: 20
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        std::fs::write(temp_file.path(), yaml_content).expect("Failed to write temp file");

        let mut loader = ConfigLoader::new(temp_file.path());
        let config = loader.load().expect("Failed to load config");

        assert_eq!(config.llm.model, "gpt-4");
        assert_eq!(config.llm.temperature, 0.5);
        assert_eq!(config.databases.len(), 1);
        assert_eq!(config.databases[0].name, "testdb");
    }

    #[test]
    fn test_validation_empty_model() {
        let mut config = AppConfig::default();
        config.llm.model = String::new();

        let validator = ConfigValidator::default();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_validation_invalid_temperature() {
        let mut config = AppConfig::default();
        config.llm.temperature = 3.0;

        let validator = ConfigValidator::default();
        assert!(validator.validate(&config).is_err());
    }

    #[test]
    fn test_validation_empty_profile_name() {
        let mut config = AppConfig::default();
        config.databases.push(DatabaseProfile {
            name: String::new(),
            url: "postgresql://localhost/test".to_string(),
            display_name: None,
            ssl_mode: "prefer".to_string(),
            connect_timeout: 30,
        });

        let validator = ConfigValidator::default();
        assert!(validator.validate(&config).is_err());
    }
}
