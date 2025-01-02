use std::sync::Arc;

use forge_provider::{CompletionMessage, ProviderService, Request, ResultStream};
use tokio_stream::StreamExt;

use super::Service;
use crate::app::{ChatRequest, ChatResponse};
use crate::Error;

#[async_trait::async_trait]
pub trait NeoChatService {
    async fn chat(&self, request: ChatRequest) -> ResultStream<ChatResponse, Error>;
}

impl Service {
    pub fn neo_chat_service(provider: Arc<dyn ProviderService>) -> impl NeoChatService {
        Live::new(provider)
    }
}

struct Live {
    provider: Arc<dyn ProviderService>,
}

impl Live {
    fn new(provider: Arc<dyn ProviderService>) -> Self {
        Self { provider }
    }
}

#[async_trait::async_trait]
impl NeoChatService for Live {
    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error> {
        let request = Request::default()
            .add_message(CompletionMessage::user(chat.content))
            .model(chat.model);
        let response = self.provider.chat(request).await?;

        let stream = response.map(|message| match message {
            Ok(message) => Ok(ChatResponse::Text(message.content)),
            Err(error) => Ok(ChatResponse::Fail((&Error::from(error)).into())),
        });

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use derive_setters::Setters;
    use forge_provider::{
        Error, Model, ModelId, Parameters, ProviderError, ProviderService, Request, Response,
        Result, ResultStream,
    };
    use pretty_assertions::assert_eq;
    use serde_json::json;
    
    use tokio_stream::StreamExt;

    use super::Live;
    use crate::app::{ChatRequest, ChatResponse};
    use crate::service::neo_chat_service::NeoChatService;

    #[derive(Default, Setters)]
    struct TestProvider {
        messages: Vec<Response>,
        request: Mutex<Option<Request>>,
        models: Vec<Model>,
        parameters: HashMap<ModelId, Parameters>,
    }

    #[async_trait::async_trait]
    impl ProviderService for TestProvider {
        async fn chat(&self, request: Request) -> ResultStream<Response, Error> {
            self.request.lock().unwrap().replace(request);
            Ok(Box::pin(tokio_stream::iter(self.messages.clone()).map(Ok)))
        }

        async fn models(&self) -> Result<Vec<Model>> {
            Ok(self.models.clone())
        }

        async fn parameters(&self, model: ModelId) -> Result<Parameters> {
            match self.parameters.get(&model) {
                None => Err(forge_provider::Error::Provider {
                    provider: "closed_ai".to_string(),
                    error: ProviderError::UpstreamError(json!({"error": "Model not found"})),
                }),
                Some(parameter) => Ok(parameter.clone()),
            }
        }
    }

    #[tokio::test]
    async fn test_chat_request() {
        let message = "Sure thing!".to_string();
        let provider =
            Arc::new(TestProvider::default().messages(vec![Response::assistant(message.clone())]));
        let service = Live::new(provider.clone());

        let chat_request = ChatRequest::new("Hello can you help me?");

        let actual = service
            .chat(chat_request)
            .await
            .unwrap()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .map(|value| value.unwrap())
            .collect::<Vec<_>>();

        let expected = vec![ChatResponse::Text(message)];

        assert_eq!(actual, expected)
    }
}
