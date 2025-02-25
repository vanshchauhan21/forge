use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use forge_domain::{Point, Query, Suggestion, SuggestionService};

use crate::{EmbeddingService, Infrastructure, VectorIndex};

pub struct ForgeSuggestionService<F> {
    infra: Arc<F>,
}

impl<F: Infrastructure> ForgeSuggestionService<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self { infra }
    }
}

#[async_trait]
impl<F: Infrastructure> SuggestionService for ForgeSuggestionService<F> {
    async fn search(&self, query: &str) -> Result<Vec<Suggestion>> {
        let embeddings = self.infra.embedding_service().embed(query).await?;
        let suggestions = self
            .infra
            .vector_index()
            .search(Query::new(embeddings).limit(5u64))
            .await?;
        Ok(suggestions.into_iter().map(|p| p.content).collect())
    }

    async fn insert(&self, suggestion: Suggestion) -> Result<()> {
        let embeddings = self
            .infra
            .embedding_service()
            .embed(&suggestion.use_case)
            .await?;

        let point = Point::new(suggestion, embeddings);
        self.infra.vector_index().store(point).await
    }
}
