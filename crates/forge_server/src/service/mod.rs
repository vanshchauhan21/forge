mod completion_service;
mod neo_chat_service;
mod root_api_service;
mod system_prompt_service;
pub use completion_service::*;
pub use neo_chat_service::*;
pub use root_api_service::*;
pub use system_prompt_service::*;

pub struct Service;

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use derive_setters::Setters;
    use forge_provider::{
        Model, ModelId, Parameters, ProviderError, ProviderService, Request, Response, ResultStream,
    };
    use serde_json::json;
    use tokio_stream::StreamExt;

    use super::SystemPromptService;
    use crate::Result;

    pub struct TestSystemPrompt {
        prompt: String,
    }

    impl TestSystemPrompt {
        pub fn new(s: impl ToString) -> Self {
            Self { prompt: s.to_string() }
        }
    }

    #[async_trait::async_trait]
    impl SystemPromptService for TestSystemPrompt {
        async fn get_system_prompt(&self, _: &ModelId) -> Result<String> {
            Ok(self.prompt.to_string())
        }
    }

    #[derive(Default, Setters)]
    pub struct TestProvider {
        messages: Vec<Response>,
        request: Mutex<Option<Request>>,
        models: Vec<Model>,
        parameters: Vec<(ModelId, Parameters)>,
    }

    impl TestProvider {
        pub fn get_last_call(&self) -> Option<Request> {
            self.request.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl ProviderService for TestProvider {
        async fn chat(&self, request: Request) -> ResultStream<Response, forge_provider::Error> {
            self.request.lock().unwrap().replace(request);
            // TODO: don't remove this else tests stop working, but we need to understand
            // why so revisit this later on.
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            Ok(Box::pin(tokio_stream::iter(self.messages.clone()).map(Ok)))
        }

        async fn models(&self) -> forge_provider::Result<Vec<Model>> {
            Ok(self.models.clone())
        }

        async fn parameters(&self, model: &ModelId) -> forge_provider::Result<Parameters> {
            match self.parameters.iter().find(|(id, _)| id == model) {
                None => Err(forge_provider::Error::Provider {
                    provider: "closed_ai".to_string(),
                    error: ProviderError::UpstreamError(json!({"error": "Model not found"})),
                }),
                Some((_, parameter)) => Ok(parameter.clone()),
            }
        }
    }
}
