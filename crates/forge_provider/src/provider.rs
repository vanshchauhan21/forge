use anyhow::Result;
use forge_domain::{
    self, ChatCompletionMessage, Context as ChatContext, Model, ModelId, Parameters, ResultStream,
};
use moka2::future::Cache;

#[async_trait::async_trait]
pub trait ProviderService: Send + Sync + 'static {
    async fn chat(
        &self,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error>;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn parameters(&self, model: &ModelId) -> Result<Parameters>;
}

pub(crate) struct Live {
    provider: Box<dyn ProviderService>,
    cache: Cache<ModelId, Parameters>,
}

impl Live {
    pub(crate) fn new(provider: impl ProviderService + 'static) -> Self {
        Self { provider: Box::new(provider), cache: Cache::new(1024) }
    }
}

#[async_trait::async_trait]
impl ProviderService for Live {
    async fn chat(
        &self,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        self.provider.chat(request).await
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
