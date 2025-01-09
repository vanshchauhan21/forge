use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use super::Environment;

#[derive(Clone, Debug, Deserialize, Serialize, Setters)]
pub struct Model {
    pub id: ModelId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Parameters {
    pub tool_supported: bool,
}

impl Parameters {
    pub fn new(tool_supported: bool) -> Self {
        Self { tool_supported }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Hash, Eq)]
#[serde(transparent)]
pub struct ModelId(String);

impl ModelId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }
}

impl ModelId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ModelId {
    fn default() -> Self {
        ModelId("openai/gpt-3.5-turbo".to_string())
    }
}

impl ModelId {
    pub fn from_env(env: &Environment) -> Self {
        ModelId(env.large_model_id.clone())
    }
}
