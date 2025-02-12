use std::path::PathBuf;

use derive_setters::Setters;

#[derive(Debug, serde::Deserialize, Clone, Setters)]
#[setters(into, strip_option)]
pub struct ChatRequest {
    pub content: String,
    pub custom_instructions: Option<PathBuf>,
}

impl ChatRequest {
    pub fn new(content: impl ToString) -> Self {
        Self { content: content.to_string(), custom_instructions: None }
    }
}
