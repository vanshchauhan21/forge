use derive_setters::Setters;

use crate::error::Result;
use crate::provider::{InnerProvider, Provider};
use crate::ResultStream;

#[derive(Clone, Setters)]
pub struct Mock<Request, Response, Error> {
    models: Vec<String>,
    events: std::result::Result<Vec<std::result::Result<Response, Error>>, Error>,
    _request: std::marker::PhantomData<Request>,
}

#[async_trait::async_trait]
impl<
        Request: Send + Sync + 'static,
        Response: Clone + Send + Sync + 'static,
        Error: Clone + Send + Sync + 'static,
    > InnerProvider for Mock<Request, Response, Error>
{
    type Request = Request;
    type Response = Response;
    type Error = Error;

    async fn chat(&self, _: Self::Request) -> ResultStream<Self::Response, Self::Error> {
        let events = self.events.clone()?;

        let stream = tokio_stream::iter(events);
        Ok(Box::pin(stream))
    }

    async fn models(&self) -> Result<Vec<String>> {
        Ok(self.models.clone())
    }
}

impl<
        Request: Clone + Send + Sync + 'static,
        Response: Clone + Send + Sync + 'static,
        Error: Clone + Send + Sync + 'static,
    > Mock<Request, Response, Error>
{
    pub fn into_provider(self) -> Provider<Request, Response, Error> {
        Provider::new(self)
    }
}
