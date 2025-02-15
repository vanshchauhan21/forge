use derive_setters::Setters;

use crate::ConversationId;

#[derive(Debug, serde::Deserialize, Clone, Setters)]
#[setters(into, strip_option)]
pub struct ChatRequest {
    pub content: String,
    pub conversation_id: ConversationId,
}

impl ChatRequest {
    pub fn new(content: impl ToString, conversation_id: ConversationId) -> Self {
        Self { content: content.to_string(), conversation_id }
    }
}
