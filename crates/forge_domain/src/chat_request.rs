use std::path::PathBuf;

use derive_setters::Setters;

use crate::{ConversationId, ModelId};

#[derive(Debug, serde::Deserialize, Clone, Setters)]
#[setters(into, strip_option)]
pub struct ChatRequest {
    pub content: String,
    pub model: ModelId,
    pub conversation_id: Option<ConversationId>,
    pub custom_instructions: Option<PathBuf>,
}

impl ChatRequest {
    pub fn new(model: ModelId, content: impl ToString) -> Self {
        Self {
            model,
            content: content.to_string(),
            conversation_id: None,
            custom_instructions: None,
        }
    }
}
