use std::sync::Arc;

use anyhow::{Context, Result};
use forge_domain::{
    ChatCompletionMessage, Context as ChatContext, Model, ModelId, Parameters, ProviderService,
    ResultStream,
};
use forge_open_router::{Client, ClientBuilder};
use moka2::future::Cache;
use tokio::sync::Mutex;

use crate::{CredentialRepository, EnvironmentService, Infrastructure};

pub struct ForgeProviderService<F> {
    infra: Arc<F>,

    // The provider service implementation
    client: Mutex<Option<Arc<Client>>>,
    cache: Cache<ModelId, Parameters>,
}

impl<F: Infrastructure> ForgeProviderService<F> {
    pub fn new(infra: Arc<F>) -> Self {
        let infra = infra.clone();
        Self { infra, client: Mutex::new(None), cache: Cache::new(1024) }
    }

    async fn provider(&self) -> Result<Arc<Client>> {
        let mut guard = self.client.lock().await;
        if let Some(provider) = guard.as_ref() {
            return Ok(provider.clone());
        }

        let env = self.infra.environment_service().get_environment();
        let key = if env.force_antinomy.unwrap_or_default() {
            self.infra
                .credential_repository()
                .credentials()
                .ok_or_else(|| anyhow::anyhow!("Failed to authenticate the user"))?
        } else {
            env.provider_key.clone()
        };
        let provider = Arc::new(
            ClientBuilder::from_url(env.provider_url)
                .api_key(key)
                .build()?,
        );

        *guard = Some(provider.clone());
        Ok(provider)
    }
}

#[async_trait::async_trait]
impl<F: Infrastructure> ProviderService for ForgeProviderService<F> {
    async fn chat(
        &self,
        model_id: &ModelId,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        self.provider()
            .await?
            .chat(model_id, request)
            .await
            .with_context(|| format!("Failed to chat with model: {}", model_id))
    }

    async fn models(&self) -> Result<Vec<Model>> {
        self.provider().await?.models().await
    }

    async fn parameters(&self, model: &ModelId) -> anyhow::Result<Parameters> {
        self.cache
            .try_get_with_by_ref(model, async {
                self.provider()
                    .await?
                    .parameters(model)
                    .await
                    .with_context(|| format!("Failed to get parameters for model: {}", model))
            })
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }
}
