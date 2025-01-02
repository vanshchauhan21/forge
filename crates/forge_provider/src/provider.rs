use derive_setters::Setters;
use moka2::future::Cache;
use serde::{Deserialize, Serialize};

use super::error::Result;
use crate::{Error, Model, ModelId, Request, Response, ResultStream};

#[async_trait::async_trait]
pub trait ProviderService: Send + Sync + 'static {
    async fn chat(&self, request: Request) -> ResultStream<Response, Error>;
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
    async fn chat(&self, request: Request) -> ResultStream<Response, Error> {
        self.provider.chat(request).await
    }

    async fn models(&self) -> Result<Vec<Model>> {
        self.provider.models().await
    }

    async fn parameters(&self, model: &ModelId) -> Result<Parameters> {
        let parameters = self
            .cache
            .try_get_with_by_ref(model, self.provider.parameters(model))
            .await;

        Ok(parameters?)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Setters)]
pub struct Parameters {
    pub tool_supported: bool,
    pub model: ModelId,
}
