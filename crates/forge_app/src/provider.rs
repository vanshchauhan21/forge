use std::sync::Arc;

use anyhow::{Context, Result};
use forge_domain::{
    ChatCompletionMessage, Context as ChatContext, Model, ModelId, Parameters, ProviderService,
    ResultStream,
};
use forge_open_router::Client;
use moka2::future::Cache;

use crate::{EnvironmentService, Infrastructure};

pub struct ForgeProviderService {
    // The provider service implementation
    client: Client,
    cache: Cache<ModelId, Parameters>,
}

impl ForgeProviderService {
    pub fn new<F: Infrastructure>(infra: Arc<F>) -> Self {
        let infra = infra.clone();
        let provider = infra
            .environment_service()
            .get_environment()
            .provider
            .clone();
        Self {
            client: Client::new(provider).unwrap(),
            cache: Cache::new(1024),
        }
    }
}

#[async_trait::async_trait]
impl ProviderService for ForgeProviderService {
    async fn chat(
        &self,
        model_id: &ModelId,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        self.client
            .chat(model_id, request)
            .await
            .with_context(|| format!("Failed to chat with model: {}", model_id))
    }

    async fn models(&self) -> Result<Vec<Model>> {
        self.client.models().await
    }

    async fn parameters(&self, model: &ModelId) -> anyhow::Result<Parameters> {
        self.cache
            .try_get_with_by_ref(model, async {
                self.client
                    .parameters(model)
                    .await
                    .with_context(|| format!("Failed to get parameters for model: {}", model))
            })
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
}
