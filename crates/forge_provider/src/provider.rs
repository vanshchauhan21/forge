use super::error::Result;
use crate::model::{Request, Response};
use crate::{Error, ResultStream};

#[async_trait::async_trait]
pub(crate) trait InnerProvider: Send + Sync + 'static {
    async fn chat(&self, request: Request) -> ResultStream<Response, Error>;
    async fn models(&self) -> Result<Vec<String>>;
}

pub struct Provider {
    provider: Box<dyn InnerProvider>,
}

impl Provider {
    pub async fn chat(&self, request: Request) -> ResultStream<Response, Error> {
        self.provider.chat(request).await
    }

    pub async fn models(&self) -> Result<Vec<String>> {
        self.provider.models().await
    }

    pub(crate) fn new(provider: impl InnerProvider + 'static) -> Self {
        Self { provider: Box::new(provider) }
    }
}
