use derive_setters::Setters;

use crate::{ConversationId, ModelId};

#[derive(Debug, serde::Deserialize, Clone, Setters)]
#[setters(into)]
pub struct ChatRequest {
    pub content: String,
    pub model: ModelId,
    pub conversation_id: Option<ConversationId>,
}

impl ChatRequest {
    pub fn new(model: ModelId, content: impl ToString) -> Self {
        Self { model, content: content.to_string(), conversation_id: None }
    }
}
