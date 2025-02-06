use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use forge_domain::Environment;

use crate::info::Info;
use crate::model::ConfigKey;

impl From<&Config> for Info {
    fn from(config: &Config) -> Self {
        let mut info = Info::new().add_title("Configuration");
        if config.is_empty() {
            info = info.add_item("Status", "No configurations set");
        } else {
            let mut configs: Vec<_> = config.values.iter().collect();
            configs.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str())); // Sort by key string
            for (key, value) in configs {
                info = info.add_item(key.as_str(), value.as_str());
            }
        }
        info
    }
}

/// Custom error type for configuration-related errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Model name cannot be empty")]
    EmptyModelName,
    #[error("Tool timeout must be greater than zero")]
    NonPositiveTimeout,
    #[error("Failed to parse timeout value: {0}")]
    MalformedTimeout(String),
}

/// Represents configuration values with their specific types
#[derive(Debug, Clone)]
pub enum ConfigValue {
    /// Model identifier string
    Model(String),
    /// Tool timeout in seconds
    ToolTimeout(u32),
}

impl ConfigValue {
    /// Returns the string representation of the configuration value
    pub fn as_str(&self) -> String {
        match self {
            ConfigValue::Model(model) => model.clone(),
            ConfigValue::ToolTimeout(timeout) => timeout.to_string(),
        }
    }

    /// Creates a new ConfigValue from a key-value pair
    pub fn from_key_value(key: &ConfigKey, value: &str) -> Result<Self, ConfigError> {
        match key {
            ConfigKey::PrimaryModel | ConfigKey::SecondaryModel => {
                if value.trim().is_empty() {
                    Err(ConfigError::EmptyModelName)
                } else {
                    Ok(ConfigValue::Model(value.to_string()))
                }
            }
            ConfigKey::ToolTimeout => match value.parse::<u32>() {
                Ok(0) => Err(ConfigError::NonPositiveTimeout),
                Ok(timeout) => Ok(ConfigValue::ToolTimeout(timeout)),
                Err(_) => Err(ConfigError::MalformedTimeout(value.to_string())),
            },
        }
    }
}

impl Display for ConfigValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Main configuration structure holding all config values
#[derive(Default)]
pub struct Config {
    values: HashMap<ConfigKey, ConfigValue>,
}

impl From<&Environment> for Config {
    fn from(env: &Environment) -> Self {
        let mut config = Config::default();
        // No need to handle errors here as we control the input values
        let _ = config.insert(&ConfigKey::PrimaryModel, &env.large_model_id);
        let _ = config.insert(&ConfigKey::SecondaryModel, &env.small_model_id);
        let _ = config.insert(&ConfigKey::ToolTimeout, "20");
        config
    }
}

impl Config {
    /// Returns the primary model configuration if set
    pub fn primary_model(&self) -> Option<String> {
        self.get_model(&ConfigKey::PrimaryModel)
    }

    /// Helper method to get model configuration
    fn get_model(&self, key: &ConfigKey) -> Option<String> {
        self.values.get(key).and_then(|v| match v {
            ConfigValue::Model(m) => Some(m.clone()),
            _ => None,
        })
    }

    /// Gets a configuration value by key string
    pub fn get(&self, key: &ConfigKey) -> Option<String> {
        self.values.get(key).map(|v| v.as_str())
    }

    /// Inserts a new configuration value
    pub fn insert(&mut self, key: &ConfigKey, value: &str) -> Result<(), ConfigError> {
        let config_value = ConfigValue::from_key_value(key, value)?;
        self.values.insert(key.clone(), config_value);
        Ok(())
    }

    /// Checks if the configuration is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_basic() {
        let mut config = Config::default();
        assert!(config.is_empty());

        // Test setting and getting values
        config.insert(&ConfigKey::PrimaryModel, "gpt-4").unwrap();
        assert_eq!(config.get(&ConfigKey::PrimaryModel).unwrap(), "gpt-4");

        config.insert(&ConfigKey::ToolTimeout, "30").unwrap();
        assert_eq!(config.get(&ConfigKey::ToolTimeout).unwrap(), "30");

        // Test type-safe accessors
        assert_eq!(config.primary_model().unwrap(), "gpt-4");

        // Test overwriting values
        config
            .insert(&ConfigKey::PrimaryModel, "gpt-3.5-turbo")
            .unwrap();
        assert_eq!(config.primary_model().unwrap(), "gpt-3.5-turbo");

        // Test getting non-existent key
        assert!(config.get(&ConfigKey::SecondaryModel).is_none());

        // Test invalid operations
        assert!(matches!(
            config
                .insert(&ConfigKey::ToolTimeout, "invalid")
                .unwrap_err(),
            ConfigError::MalformedTimeout(_)
        ));
        assert!(matches!(
            config.insert(&ConfigKey::ToolTimeout, "0").unwrap_err(),
            ConfigError::NonPositiveTimeout
        ));
    }
}
