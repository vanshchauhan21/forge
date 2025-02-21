use std::sync::Arc;

use forge_domain::{ExecutableTool, Knowledge, NamedTool, Query, ToolDescription, ToolName};
use schemars::JsonSchema;
use serde_json::json;

use crate::{EmbeddingService, Infrastructure, KnowledgeRepository};

pub struct RecallKnowledge<F> {
    infra: Arc<F>,
}

impl<F> ToolDescription for RecallKnowledge<F> {
    fn description(&self) -> String {
        "Get knowledge from the app".to_string()
    }
}

impl<F> RecallKnowledge<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self { infra }
    }
}

#[derive(serde::Deserialize, JsonSchema)]
pub struct GetKnowledgeInput {
    pub query: String,
}

#[async_trait::async_trait]
impl<F: Infrastructure> ExecutableTool for RecallKnowledge<F> {
    type Input = GetKnowledgeInput;

    async fn call(&self, input: Self::Input) -> anyhow::Result<String> {
        let embedding = self.infra.embedding_service().embed(&input.query).await?;
        let out = self
            .infra
            .textual_knowledge_repo()
            .search(Query::new(embedding))
            .await?
            .into_iter()
            .map(|k| serde_json::to_string(&k))
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");

        Ok(out)
    }
}

impl<F> NamedTool for RecallKnowledge<F> {
    fn tool_name() -> ToolName {
        ToolName::new("forge_tool_knowledge_get".to_string())
    }
}

pub struct StoreKnowledge<F> {
    infra: Arc<F>,
}

impl<F> StoreKnowledge<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self { infra }
    }
}

impl<F> ToolDescription for StoreKnowledge<F> {
    fn description(&self) -> String {
        "Set knowledge to the app".to_string()
    }
}

#[derive(serde::Deserialize, JsonSchema)]
pub struct StoreKnowledgeInput {
    pub content: String,
}

#[async_trait::async_trait]
impl<F: Infrastructure> ExecutableTool for StoreKnowledge<F> {
    type Input = StoreKnowledgeInput;

    async fn call(&self, input: Self::Input) -> anyhow::Result<String> {
        let embedding = self.infra.embedding_service().embed(&input.content).await?;
        let knowledge = Knowledge::new(json!({"content": input.content}), embedding);
        self.infra
            .textual_knowledge_repo()
            .store(vec![knowledge])
            .await?;

        Ok("Updated knowledge successfully".to_string())
    }
}

impl<F> NamedTool for StoreKnowledge<F> {
    fn tool_name() -> ToolName {
        ToolName::new("forge_tool_knowledge_set".to_string())
    }
}
