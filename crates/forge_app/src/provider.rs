use std::sync::Arc;

use anyhow::{Context, Result};
use forge_domain::{
    ChatCompletionMessage, Context as ChatContext, Model, ModelId, Parameters, ProviderService,
    ResultStream,
};
use forge_open_router::OpenRouter;
use moka2::future::Cache;

use crate::{EnvironmentService, Infrastructure};

pub struct ForgeProviderService {
    or: OpenRouter,
    cache: Cache<ModelId, Parameters>,
}

impl ForgeProviderService {
    pub fn new<F: Infrastructure>(infra: Arc<F>) -> Self {
        let or = OpenRouter::builder()
            .api_key(infra.environment_service().get_environment().api_key)
            .build()
            .unwrap();
        Self { or, cache: Cache::new(1024) }
    }
}

#[async_trait::async_trait]
impl ProviderService for ForgeProviderService {
    async fn chat(
        &self,
        model_id: &ModelId,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        self.or.chat(model_id, request).await
    }

    async fn models(&self) -> Result<Vec<Model>> {
        self.or.models().await
    }

    async fn parameters(&self, model: &ModelId) -> anyhow::Result<Parameters> {
        Ok(self
            .cache
            .try_get_with_by_ref(model, async {
                self.or
                    .parameters(model)
                    .await
                    .with_context(|| format!("Failed to get parameters for model: {}", model))
            })
            .await
            .map_err(|e| anyhow::anyhow!(e))?)
    }
}
