
use crate::error::{Error, Result};
use crate::model::{Request, Response};
use crate::provider::{InnerProvider, Provider};
use crate::ResultStream;

#[derive(Clone)]
pub struct Mock {
    model: String,
}

impl Mock {
    pub fn new(model: Option<String>) -> Self {
        Self {
            model: model.unwrap_or_else(|| "mock/default-model".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl InnerProvider for Mock {
    async fn chat(&self, request: Request) -> ResultStream<Response, Error> {
        // Create a stream that emits a single mock response
        let response = Response::new(format!(
            "Mock response using model: {}. Received {} messages.",
            self.model,
            request.context.len()
        ));

        let stream = tokio_stream::once(Ok(response));
        Ok(Box::pin(stream))
    }

    async fn models(&self) -> Result<Vec<String>> {
        Ok(vec![
            "mock/quantum-sage".to_string(),
            "mock/neural-phoenix".to_string(),
            "mock/cosmic-oracle".to_string(),
            "mock/digital-alchemist".to_string(),
            "mock/cyber-sentinel".to_string(),
        ])
    }
}

impl Provider {
    pub fn mock(model: Option<String>) -> Self {
        Provider::new(Mock::new(model))
    }
}
