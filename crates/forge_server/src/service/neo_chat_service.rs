use std::sync::Arc;

use forge_provider::{
    CompletionMessage, FinishReason, ProviderService, Request, ResultStream, ToolCall,
};
use forge_tool::ToolService;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use super::{Service, SystemPromptService};
use crate::app::{ChatRequest, ChatResponse};
use crate::{Error, Result};

#[async_trait::async_trait]
pub trait NeoChatService {
    async fn chat(&self, request: ChatRequest) -> ResultStream<ChatResponse, Error>;
}

impl Service {
    pub fn neo_chat_service(
        provider: Arc<dyn ProviderService>,
        system_prompt: Arc<dyn SystemPromptService>,
        tool: Arc<dyn ToolService>,
    ) -> impl NeoChatService {
        Live::new(provider, system_prompt, tool)
    }
}

#[derive(Clone)]
struct Live {
    provider: Arc<dyn ProviderService>,
    system_prompt: Arc<dyn SystemPromptService>,
    tool: Arc<dyn ToolService>,
}

impl Live {
    fn new(
        provider: Arc<dyn ProviderService>,
        system_prompt: Arc<dyn SystemPromptService>,
        tool: Arc<dyn ToolService>,
    ) -> Self {
        Self { provider, system_prompt, tool }
    }

    async fn loop_until(
        &self,
        mut request: Request,
        tx: tokio::sync::mpsc::Sender<Result<ChatResponse>>,
    ) -> Result<()> {
        loop {
            let mut response = self.provider.chat(request.clone()).await?;
            let mut current_tool = Vec::new();
            let mut assistant_buffer = Vec::new();
            let mut tool_result = None;

            while let Some(chunk) = response.next().await {
                let message = chunk?;
                if message.tool_call.is_empty() {
                    // TODO: drop unwrap from here.
                    assistant_buffer.push(message.clone());
                    tx.send(Ok(ChatResponse::Text(message.content)))
                        .await
                        .expect("Failed to send message");
                } else {
                    // we start the folding process.
                    if let Some(tool_part) = message.tool_call.first() {
                        current_tool.push(tool_part.clone());
                    }

                    if let Some(FinishReason::ToolCalls) = message.finish_reason {
                        // TODO: drop clone from here.
                        let actual_tool_call = ToolCall::try_from_parts(current_tool.clone())?;
                        tool_result = Some(
                            self.tool
                                .call(&actual_tool_call.name, actual_tool_call.arguments)
                                .await,
                        );
                    }
                }
            }

            let assitant_message = "".to_string();
            request = request.add_message(CompletionMessage::assistant(assitant_message));
            if let Some(Ok(tool_result)) = tool_result {
                let tool_result = serde_json::from_value(tool_result).unwrap();
                request = request.add_message(CompletionMessage::ToolMessage(tool_result));
            } else {
                break Ok(());
            }
        }
    }
}

#[async_trait::async_trait]
impl NeoChatService for Live {
    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error> {
        let system_prompt = self.system_prompt.get_system_prompt(&chat.model).await?;
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let request = Request::default()
            .set_system_message(system_prompt)
            .add_message(CompletionMessage::user(chat.content))
            .model(chat.model);

        let that = self.clone();
        tokio::spawn(async move {
            // TODO: simplify this match.
            match that.loop_until(request, tx.clone()).await {
                Ok(_) => {}
                Err(e) => tx.send(Err(e)).await.unwrap(),
            };
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::{Arc, Mutex};
    use std::vec;

    use derive_setters::Setters;
    use forge_provider::{
        CompletionMessage, Error, Model, ModelId, Parameters, ProviderError, ProviderService,
        Request, Response, Result, ResultStream,
    };
    use forge_tool::{ToolDefinition, ToolName, ToolService};
    use pretty_assertions::assert_eq;
    use serde_json::{json, Value};
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

    struct TestToolService {
        result: Value,
    }

    impl TestToolService {
        pub fn new(result: Value) -> Self {
            Self { result }
        }
    }

    #[async_trait::async_trait]
    impl ToolService for TestToolService {
        async fn call(
            &self,
            _name: &ToolName,
            _input: Value,
        ) -> std::result::Result<Value, String> {
            Ok(self.result.clone())
        }
        fn list(&self) -> Vec<ToolDefinition> {
            vec![]
        }
        fn usage_prompt(&self) -> String {
            "".to_string()
        }
    }

    #[async_trait::async_trait]
    impl ProviderService for TestProvider {
        async fn chat(&self, request: Request) -> ResultStream<Response, Error> {
            self.request.lock().unwrap().replace(request);
            // TODO: don't remove this else tests stop working, but we need to understand
            // why so revisit this later on.
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            Ok(Box::pin(
                tokio_stream::iter(self.messages.clone()).map(Ok),
            ))
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
        let tool = Arc::new(TestToolService::new(json!({"result": "fs success."})));
        let service = Live::new(provider.clone(), system_prompt, tool);

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
        let tool = Arc::new(TestToolService::new(json!({"result": "fs success."})));
        let service = Live::new(provider.clone(), system_prompt, tool);

        let chat_request = ChatRequest::new("Hello can you help me?");

        // TODO: don't remove this else tests stop working, but we need to understand
        // why so revisit this later on.
        tokio::time::pause();
        let _ = service.chat(chat_request).await.unwrap();
        tokio::time::advance(tokio::time::Duration::from_millis(5)).await;

        let actual = provider.get_last_call().unwrap().messages[0].clone();
        let expected = CompletionMessage::system(system_message);

        assert_eq!(actual, expected)
    }
}
