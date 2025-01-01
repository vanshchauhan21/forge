use moka2::future::Cache;

use super::error::Result;
use crate::{Error, Model, ModelId, Request, Response, ResultStream};

#[async_trait::async_trait]
pub(crate) trait InnerProvider: Send + Sync + 'static {
    async fn chat(&self, request: Request) -> ResultStream<Response, Error>;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn parameters(&self, model: ModelId) -> Result<Parameters>;
}

pub struct Provider {
    provider: Box<dyn InnerProvider>,
    cache: Cache<ModelId, Parameters>,
}

impl Provider {
    pub async fn chat(&self, request: Request) -> ResultStream<Response, Error> {
        self.provider.chat(request).await
    }

    pub async fn models(&self) -> Result<Vec<Model>> {
        self.provider.models().await
    }

    pub(crate) fn new(provider: impl InnerProvider + 'static) -> Self {
        Self { provider: Box::new(provider), cache: Cache::new(1024) }
    }

    pub async fn parameters(&self, model: ModelId) -> Result<Parameters> {
        let parameters = self
            .cache
            .try_get_with_by_ref(&model, self.provider.parameters(model.clone()))
            .await;

        Ok(parameters?)
    }
}

#[derive(Clone)]
pub struct Parameters {
    pub tools: bool,
}
