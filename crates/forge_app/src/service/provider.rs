use anyhow::Result;
use forge_domain::{
    ChatCompletionMessage, Context as ChatContext, Model, ModelId, Parameters, ProviderService,
    ResultStream,
};
use forge_open_router::OpenRouter;
use moka2::future::Cache;

use super::Service;

impl Service {
    pub fn provider_service(api_key: impl ToString) -> impl ProviderService {
        Live::new(api_key)
    }
}

struct Live {
    provider: Box<dyn ProviderService>,
    cache: Cache<ModelId, Parameters>,
}

impl Live {
    fn new(api_key: impl ToString) -> Self {
        let provider = OpenRouter::new(api_key);
        Self { provider: Box::new(provider), cache: Cache::new(1024) }
    }
}

#[async_trait::async_trait]
impl ProviderService for Live {
    async fn chat(
        &self,
        model_id: &ModelId,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        self.provider.chat(model_id, request).await
    }

    async fn models(&self) -> Result<Vec<Model>> {
        self.provider.models().await
    }

    async fn parameters(&self, model: &ModelId) -> Result<Parameters> {
        match self
            .cache
            .try_get_with_by_ref(model, self.provider.parameters(model))
            .await
        {
            Ok(parameters) => Ok(parameters),
            Err(e) => anyhow::bail!("Failed to get parameters from cache: {}", e),
        }
    }
}
