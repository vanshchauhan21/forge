use derive_setters::Setters;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Setters)]
pub struct Model {
    pub id: ModelId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Setters)]
pub struct Parameters {
    pub tool_supported: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Hash, Eq)]
#[serde(transparent)]
pub struct ModelId(String);

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
