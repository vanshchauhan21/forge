use chrono::{DateTime, NaiveDateTime, Utc};
use derive_setters::Setters;
use diesel::prelude::*;
use diesel::sql_types::{Bool, Text, Timestamp};
use forge_domain::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Service;
use crate::schema::conversations;
use crate::service::db_service::DBService;
use crate::Result;

#[derive(Debug, Setters, Serialize, Deserialize)]
pub struct Conversation {
    pub id: ConversationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ConversationMeta>,
    pub context: Context,
    pub archived: bool,
}

impl Conversation {
    pub fn new(context: Context) -> Self {
        Self {
            id: ConversationId::generate(),
            meta: None,
            context,
            archived: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Copy)]
#[serde(transparent)]
pub struct ConversationId(Uuid);

impl ConversationId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationMeta {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, Queryable, QueryableByName)]
#[diesel(table_name = conversations)]
struct RawConversation {
    #[diesel(sql_type = Text)]
    id: String,
    #[diesel(sql_type = Timestamp)]
    created_at: NaiveDateTime,
    #[diesel(sql_type = Timestamp)]
    updated_at: NaiveDateTime,
    #[diesel(sql_type = Text)]
    content: String,
    #[diesel(sql_type = Bool)]
    archived: bool,
}

impl TryFrom<RawConversation> for Conversation {
    type Error = crate::error::Error;

    fn try_from(raw: RawConversation) -> Result<Self> {
        Ok(Conversation {
            id: ConversationId(Uuid::parse_str(&raw.id).unwrap()),
            meta: Some(ConversationMeta {
                created_at: DateTime::from_naive_utc_and_offset(raw.created_at, Utc),
                updated_at: DateTime::from_naive_utc_and_offset(raw.updated_at, Utc),
            }),
            context: serde_json::from_str(&raw.content)?,
            archived: raw.archived,
        })
    }
}

#[async_trait::async_trait]
pub trait ConversationService: Send + Sync {
    async fn set_conversation(
        &self,
        request: &Context,
        id: Option<ConversationId>,
    ) -> Result<Conversation>;
    async fn get_conversation(&self, id: ConversationId) -> Result<Conversation>;
    async fn list_conversations(&self) -> Result<Vec<Conversation>>;
    async fn archive_conversation(&self, id: ConversationId) -> Result<Conversation>;
}

pub struct Live<P: DBService> {
    pool_service: P,
}

impl<P: DBService> Live<P> {
    pub fn new(pool_service: P) -> Self {
        Self { pool_service }
    }
}

#[async_trait::async_trait]
impl<P: DBService + Send + Sync> ConversationService for Live<P> {
    async fn set_conversation(
        &self,
        request: &Context,
        id: Option<ConversationId>,
    ) -> Result<Conversation> {
        let pool = self.pool_service.pool().await?;
        let mut conn = pool.get()?;
        let id = id.unwrap_or_else(ConversationId::generate);

        let raw = RawConversation {
            id: id.0.to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            content: serde_json::to_string(request)?,
            archived: false,
        };

        diesel::insert_into(conversations::table)
            .values(&raw)
            .on_conflict(conversations::id)
            .do_update()
            .set((
                conversations::content.eq(&raw.content),
                conversations::updated_at.eq(&raw.updated_at),
            ))
            .execute(&mut conn)?;

        let raw: RawConversation = conversations::table
            .find(id.0.to_string())
            .first(&mut conn)?;

        Conversation::try_from(raw)
    }

    async fn get_conversation(&self, id: ConversationId) -> Result<Conversation> {
        let pool = self.pool_service.pool().await?;
        let mut conn = pool.get()?;
        let raw: RawConversation = conversations::table
            .find(id.0.to_string())
            .first(&mut conn)?;

        Conversation::try_from(raw)
    }

    async fn list_conversations(&self) -> Result<Vec<Conversation>> {
        let pool = self.pool_service.pool().await?;
        let mut conn = pool.get()?;
        let raw: Vec<RawConversation> = conversations::table
            .filter(conversations::archived.eq(false))
            .load(&mut conn)?;

        raw.into_iter().map(Conversation::try_from).collect()
    }

    async fn archive_conversation(&self, id: ConversationId) -> Result<Conversation> {
        let pool = self.pool_service.pool().await?;
        let mut conn = pool.get()?;

        diesel::update(conversations::table.find(id.0.to_string()))
            .set(conversations::archived.eq(true))
            .execute(&mut conn)?;

        let raw: RawConversation = conversations::table
            .find(id.0.to_string())
            .first(&mut conn)?;

        Conversation::try_from(raw)
    }
}

impl Service {
    pub fn storage_service(database_url: &str) -> Result<impl ConversationService> {
        let pool_service = Service::db_pool_service(database_url)?;
        Ok(Live::new(pool_service))
    }
}

#[cfg(test)]
pub mod tests {
    use forge_domain::ModelId;
    use pretty_assertions::assert_eq;

    use super::super::db_service::tests::TestDbPool;
    use super::*;

    impl ConversationId {
        pub fn new(id: impl Into<String>) -> Self {
            ConversationId(Uuid::parse_str(&id.into()).unwrap())
        }
    }
    pub struct TestStorage;
    impl TestStorage {
        pub fn in_memory() -> Result<impl ConversationService> {
            let pool_service = TestDbPool::new()?;
            Ok(Live::new(pool_service))
        }
    }

    async fn setup_storage() -> Result<impl ConversationService> {
        TestStorage::in_memory()
    }

    async fn create_conversation(
        storage: &impl ConversationService,
        id: Option<ConversationId>,
    ) -> Result<Conversation> {
        let request = Context::new(ModelId::default());
        storage.set_conversation(&request, id).await
    }

    #[tokio::test]
    async fn conversation_can_be_stored_and_retrieved() {
        let storage = setup_storage().await.unwrap();
        let id = ConversationId::generate();

        let saved = create_conversation(&storage, Some(id)).await.unwrap();
        let retrieved = storage.get_conversation(id).await.unwrap();

        assert_eq!(saved.id.0, retrieved.id.0);
        assert_eq!(saved.context, retrieved.context);
    }

    #[tokio::test]
    async fn list_returns_active_conversations() {
        let storage = setup_storage().await.unwrap();

        let conv1 = create_conversation(&storage, None).await.unwrap();
        let conv2 = create_conversation(&storage, None).await.unwrap();
        let conv3 = create_conversation(&storage, None).await.unwrap();

        // Archive one conversation
        storage.archive_conversation(conv2.id).await.unwrap();

        let conversations = storage.list_conversations().await.unwrap();

        assert_eq!(conversations.len(), 2);
        assert!(conversations.iter().all(|c| !c.archived));
        assert!(conversations.iter().any(|c| c.id == conv1.id));
        assert!(conversations.iter().any(|c| c.id == conv3.id));
        assert!(conversations.iter().all(|c| c.id != conv2.id));
    }

    #[tokio::test]
    async fn archive_marks_conversation_as_archived() {
        let storage = setup_storage().await.unwrap();
        let conversation = create_conversation(&storage, None).await.unwrap();

        let archived = storage.archive_conversation(conversation.id).await.unwrap();

        assert!(archived.archived);
        assert_eq!(archived.id, conversation.id);
    }
}
