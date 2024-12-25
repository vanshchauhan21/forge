use super::error::Result;
use crate::ResultStream;

#[async_trait::async_trait]
pub(crate) trait InnerProvider: Send + Sync + 'static {
    type Request: Send + Sync + 'static;
    type Response: Send + Sync + 'static;
    type Error: Send + Sync + 'static;

    async fn chat(&self, request: Self::Request) -> ResultStream<Self::Response, Self::Error>;
    async fn models(&self) -> Result<Vec<String>>;
}

pub struct Provider<Request, Response, Error> {
    provider: Box<dyn InnerProvider<Request = Request, Response = Response, Error = Error>>,
}

impl<
        Request: Send + Sync + 'static,
        Response: Send + Sync + 'static,
        Error: Send + Sync + 'static,
    > Provider<Request, Response, Error>
{
    pub async fn chat(&self, request: Request) -> ResultStream<Response, Error> {
        self.provider.chat(request).await
    }

    pub async fn models(&self) -> Result<Vec<String>> {
        self.provider.models().await
    }

    pub(crate) fn new(
        provider: impl InnerProvider<Request = Request, Response = Response, Error = Error> + 'static,
    ) -> Self {
        Self { provider: Box::new(provider) }
    }
}
