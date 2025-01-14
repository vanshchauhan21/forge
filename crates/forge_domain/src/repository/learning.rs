use std::fmt;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LearningId(pub Uuid);

impl Default for LearningId {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl fmt::Display for LearningId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Learning {
    pub id: LearningId,
    pub content: String,
    pub context: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait LearningRepository {
    /// Get a learning entry by its ID
    async fn get_learning(&self, id: LearningId) -> anyhow::Result<Option<Learning>>;

    /// Save a new learning entry or update an existing one
    async fn save_learning(&self, learning: &Learning) -> anyhow::Result<()>;

    /// List all learning entries
    async fn list_learnings(&self) -> anyhow::Result<Vec<Learning>>;

    /// Get learning entries by context
    async fn get_learnings_by_context(&self, context: &str) -> anyhow::Result<Vec<Learning>>;
}
