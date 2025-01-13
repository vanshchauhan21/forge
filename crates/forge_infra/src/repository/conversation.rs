use anyhow::Result;
use async_trait::async_trait;
use forge_domain::{Conversation, ConversationId, ConversationRepository};

pub struct SqliteConversationRepository {
    // Implementation details will be added later
}

impl Default for SqliteConversationRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteConversationRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ConversationRepository for SqliteConversationRepository {
    async fn get_conversation(&self, _id: ConversationId) -> Result<Option<Conversation>> {
        // Implementation will be added later
        todo!()
    }

    async fn save_conversation(&self, _conversation: &Conversation) -> Result<()> {
        // Implementation will be added later
        todo!()
    }

    async fn list_conversations(&self) -> Result<Vec<Conversation>> {
        // Implementation will be added later
        todo!()
    }

    async fn archive_conversation(&self, _id: ConversationId) -> Result<()> {
        // Implementation will be added later
        todo!()
    }
}
