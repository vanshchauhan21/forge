use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::ModelId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(transparent)]
pub struct ProviderId(String);

impl ProviderId {
    pub fn new(s: impl ToString) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(transparent)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new(s: impl ToString) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    pub provider_id: ProviderId,
    pub model_id: ModelId,
    pub api_key: Option<ApiKey>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Permissions {
    pub read: bool,
    pub edit: bool,
    pub commands: bool,
    pub browser: bool,
    pub mcp: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub primary_model: ModelConfig,
    pub secondary_model: ModelConfig,
    pub permissions: Permissions,
    pub max_requests: u32,
    pub notifications: bool,
}

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    /// Get the current configuration
    async fn get(&self) -> anyhow::Result<Config>;

    /// Save a new configuration and return the saved config
    async fn set(&self, config: Config) -> anyhow::Result<Config>;
}
