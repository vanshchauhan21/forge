use std::sync::Arc;

use forge_provider::{CompletionMessage, ProviderService, Request, ResultStream};
use tokio_stream::StreamExt;

use super::{Service, SystemPromptService};
use crate::app::{ChatRequest, ChatResponse};
use crate::Error;

#[async_trait::async_trait]
pub trait NeoChatService {
    async fn chat(&self, request: ChatRequest) -> ResultStream<ChatResponse, Error>;
}

impl Service {
    pub fn neo_chat_service(
        provider: Arc<dyn ProviderService>,
        system_prompt: Arc<dyn SystemPromptService>,
    ) -> impl NeoChatService {
        Live::new(provider, system_prompt)
    }
}

struct Live {
    provider: Arc<dyn ProviderService>,
    system_prompt: Arc<dyn SystemPromptService>,
}

impl Live {
    fn new(
        provider: Arc<dyn ProviderService>,
        system_prompt: Arc<dyn SystemPromptService>,
    ) -> Self {
        Self { provider, system_prompt }
    }
}

#[async_trait::async_trait]
impl NeoChatService for Live {
    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error> {
        let system_prompt = self.system_prompt.get_system_prompt(&chat.model).await?;

        let request = Request::default()
            .set_system_message(system_prompt)
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
pub mod tests {
    use std::sync::{Arc, Mutex};

    use derive_setters::Setters;
    use forge_provider::{
        CompletionMessage, Error, Model, ModelId, Parameters, ProviderError, ProviderService,
        Request, Response, Result, ResultStream,
    };
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use tokio_stream::StreamExt;

    use super::Live;
    use crate::app::{ChatRequest, ChatResponse};
    use crate::service::neo_chat_service::NeoChatService;
    use crate::tests::TestSystemPrompt;

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
        async fn chat(&self, request: Request) -> ResultStream<Response, Error> {
            self.request.lock().unwrap().replace(request);
            Ok(Box::pin(tokio_stream::iter(self.messages.clone()).map(Ok)))
        }

        async fn models(&self) -> Result<Vec<Model>> {
            Ok(self.models.clone())
        }

        async fn parameters(&self, model: &ModelId) -> Result<Parameters> {
            match self.parameters.iter().find(|(id, _)| id == model) {
                None => Err(forge_provider::Error::Provider {
                    provider: "closed_ai".to_string(),
                    error: ProviderError::UpstreamError(json!({"error": "Model not found"})),
                }),
                Some((_, parameter)) => Ok(parameter.clone()),
            }
        }
    }

    #[tokio::test]
    async fn test_chat_response() {
        let message = "Sure thing!".to_string();
        let provider =
            Arc::new(TestProvider::default().messages(vec![Response::assistant(message.clone())]));
        let system_prompt = Arc::new(TestSystemPrompt::new("Do everything that the user says"));
        let service = Live::new(provider.clone(), system_prompt);

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

    #[tokio::test]
    async fn test_chat_system_prompt() {
        let message = "Sure thing!".to_string();
        let provider =
            Arc::new(TestProvider::default().messages(vec![Response::assistant(message.clone())]));
        let system_message = "Do everything that the user says";
        let system_prompt = Arc::new(TestSystemPrompt::new(system_message));
        let service = Live::new(provider.clone(), system_prompt);

        let chat_request = ChatRequest::new("Hello can you help me?");

        let _ = service.chat(chat_request).await.unwrap();

        let actual = provider.get_last_call().unwrap().messages[0].clone();
        let expected = CompletionMessage::system(system_message);

        assert_eq!(actual, expected)
    }
}
