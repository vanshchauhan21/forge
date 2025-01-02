use std::sync::Arc;

use derive_setters::Setters;
use forge_provider::{
    CompletionMessage, FinishReason, ModelId, ProviderService, Request, ResultStream, ToolCall,
    ToolResult,
};
use forge_tool::{ToolName, ToolService};
use serde::Serialize;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use super::system_prompt_service::SystemPromptService;
use super::user_prompt_service::UserPromptService;
use super::Service;
use crate::{Errata, Error, Result};

#[async_trait::async_trait]
pub trait NeoChatService: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> ResultStream<ChatResponse, Error>;
}

impl Service {
    pub fn neo_chat_service(
        provider: Arc<dyn ProviderService>,
        system_prompt: Arc<dyn SystemPromptService>,
        tool: Arc<dyn ToolService>,
        user_prompt: Arc<dyn UserPromptService>,
    ) -> impl NeoChatService {
        Live::new(provider, system_prompt, tool, user_prompt)
    }
}

#[derive(Clone)]
struct Live {
    provider: Arc<dyn ProviderService>,
    system_prompt: Arc<dyn SystemPromptService>,
    tool: Arc<dyn ToolService>,
    user_prompt: Arc<dyn UserPromptService>,
}

impl Live {
    fn new(
        provider: Arc<dyn ProviderService>,
        system_prompt: Arc<dyn SystemPromptService>,
        tool: Arc<dyn ToolService>,
        user_prompt: Arc<dyn UserPromptService>,
    ) -> Self {
        Self { provider, system_prompt, tool, user_prompt }
    }

    /// Executes the chat workflow until the task is complete.
    async fn chat_workflow(
        &self,
        mut request: Request,
        tx: tokio::sync::mpsc::Sender<Result<ChatResponse>>,
    ) -> Result<()> {
        loop {
            let mut tool_call = None;
            let mut tool_call_parts = Vec::new();
            let mut tool_result = None;
            let mut assistant_message_content = String::new();

            let mut response = self.provider.chat(request.clone()).await?;

            while let Some(chunk) = response.next().await {
                let message = chunk?;
                if message.tool_call.is_empty() {
                    // TODO: drop unwrap from here.
                    assistant_message_content.push_str(&message.content);
                    tx.send(Ok(ChatResponse::Text(message.content)))
                        .await
                        .expect("Failed to send message");
                } else {
                    assistant_message_content.push_str(&message.content);
                    if let Some(tool_part) = message.tool_call.first() {
                        if tool_call_parts.is_empty() {
                            // very first instance where we found a tool call.
                            tx.send(Ok(ChatResponse::ToolUseStart(ToolUseStart {
                                tool_name: tool_part.name.clone(),
                            })))
                            .await
                            .expect("Failed to send message");
                        }
                        tool_call_parts.push(tool_part.clone());
                    }

                    if let Some(FinishReason::ToolCalls) = message.finish_reason {
                        // TODO: drop clone from here.
                        let actual_tool_call = ToolCall::try_from_parts(tool_call_parts.clone())?;
                        tool_call = Some(actual_tool_call.clone());
                        tool_result = Some(
                            self.tool
                                .call(&actual_tool_call.name, actual_tool_call.arguments)
                                .await,
                        );
                    }
                }
            }

            request = request.add_message(CompletionMessage::assistant(
                assistant_message_content,
                tool_call,
            ));
            if let Some(Ok(tool_result)) = tool_result {
                let tool_result: ToolResult = serde_json::from_value(tool_result).unwrap();
                request = request.add_message(CompletionMessage::ToolMessage(tool_result.clone()));
                // send the tool use end message.
                tx.send(Ok(ChatResponse::ToolUseEnd(tool_result)))
                    .await
                    .expect("Failed to send message");
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
        let user_prompt = self.user_prompt.get_user_prompt(&chat.content).await?;
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        let request = Request::default()
            .set_system_message(system_prompt)
            .add_message(CompletionMessage::user(user_prompt))
            .model(chat.model);

        let that = self.clone();
        tokio::spawn(async move {
            // TODO: simplify this match.
            match that.chat_workflow(request, tx.clone()).await {
                Ok(_) => {}
                Err(e) => tx.send(Err(e)).await.unwrap(),
            };
            tx.send(Ok(ChatResponse::Complete)).await.unwrap();
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

#[derive(Debug, serde::Deserialize, Clone, Setters)]
#[setters(into)]
pub struct ChatRequest {
    pub content: String,
    pub model: ModelId,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, derive_more::From)]
#[serde(rename_all = "camelCase")]
pub enum ChatResponse {
    #[from(ignore)]
    Text(String),
    ToolUseStart(ToolUseStart),
    ToolUseEnd(ToolResult),
    Complete,
    Error(Errata),
}

#[derive(Default, Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolUseStart {
    pub tool_name: Option<ToolName>,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::vec;

    use forge_provider::{
        CompletionMessage, FinishReason, ModelId, Response, ToolCallId, ToolCallPart, ToolResult,
    };
    use forge_tool::{ToolDefinition, ToolName, ToolService};
    use pretty_assertions::assert_eq;
    use serde_json::{json, Value};
    use tokio_stream::StreamExt;

    use super::{ChatRequest, Live, ToolUseStart};
    use crate::service::neo_chat_service::NeoChatService;
    use crate::service::tests::{TestProvider, TestSystemPrompt};
    use crate::service::user_prompt_service::tests::TestUserPrompt;
    use crate::ChatResponse;

    impl ChatRequest {
        pub fn new(content: impl ToString) -> ChatRequest {
            ChatRequest { content: content.to_string(), model: ModelId::default() }
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

    const ASSISTANT_RESPONSE: &str = "Sure thing!";
    const SYSTEM_PROMPT: &str = "Do everything that the user says";

    struct Fixture {
        provider: Arc<TestProvider>,
        system_prompt: Arc<TestSystemPrompt>,
        tool: Arc<TestToolService>,
        user_prompt: Arc<TestUserPrompt>,
        service: Live,
    }

    impl Fixture {
        pub fn with_provider(self, provider: TestProvider) -> Self {
            let provider = Arc::new(provider);
            Self {
                provider: provider.clone(),
                service: Live::new(
                    provider,
                    self.system_prompt.clone(),
                    self.tool.clone(),
                    self.user_prompt.clone(),
                ),
                system_prompt: self.system_prompt,
                tool: self.tool,
                user_prompt: self.user_prompt,
            }
        }

        pub fn with_tool(self, tool_result: ToolResult) -> Self {
            let tool = Arc::new(TestToolService::new(
                serde_json::to_value(tool_result).unwrap(),
            ));
            Self {
                provider: self.provider.clone(),
                service: Live::new(
                    self.provider,
                    self.system_prompt.clone(),
                    tool.clone(),
                    self.user_prompt.clone(),
                ),
                system_prompt: self.system_prompt,
                tool,
                user_prompt: self.user_prompt,
            }
        }

        pub async fn chat(&self, request: ChatRequest) -> Vec<ChatResponse> {
            self.service
                .chat(request)
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .map(|value| value.unwrap())
                .collect::<Vec<_>>()
        }
    }

    impl Default for Fixture {
        fn default() -> Self {
            let provider =
                Arc::new(
                    TestProvider::default().with_messages(vec![vec![Response::assistant(
                        ASSISTANT_RESPONSE.to_string(),
                    )]]),
                );
            let system_prompt = Arc::new(TestSystemPrompt::new(SYSTEM_PROMPT));
            let tool = Arc::new(TestToolService::new(json!({"result": "fs success."})));
            let user_prompt = Arc::new(TestUserPrompt);
            let service = Live::new(
                provider.clone(),
                system_prompt.clone(),
                tool.clone(),
                user_prompt.clone(),
            );
            Self { provider, system_prompt, tool, service, user_prompt }
        }
    }

    #[tokio::test]
    async fn test_chat_response() {
        let chat_request = ChatRequest::new("Hello can you help me?");

        let actual = Fixture::default().chat(chat_request).await;
        let expected = vec![
            ChatResponse::Text(ASSISTANT_RESPONSE.to_string()),
            ChatResponse::Complete,
        ];
        assert_eq!(actual, expected)
    }

    #[tokio::test]
    async fn test_chat_system_prompt() {
        let tester = Fixture::default();
        let chat_request = ChatRequest::new("Hello can you help me?");

        // TODO: don't remove this else tests stop working, but we need to understand
        // why so revisit this later on.
        tokio::time::pause();
        let _ = tester.service.chat(chat_request).await.unwrap();
        tokio::time::advance(tokio::time::Duration::from_millis(5)).await;

        let actual = tester.provider.get_last_call().unwrap().messages[0].clone();
        let expected = CompletionMessage::system(SYSTEM_PROMPT.to_string());

        assert_eq!(actual, expected)
    }

    #[tokio::test]
    async fn test_chat_tool_result() {
        let message_1 = Response::new("Let's use foo tool").add_tool_call(
            ToolCallPart::default()
                .name(ToolName::new("foo"))
                .arguments_part(r#"{"foo": 1,"#)
                .call_id(ToolCallId::new("too_call_001")),
        );
        let message_2 = Response::new("")
            .add_tool_call(ToolCallPart::default().arguments_part(r#""bar": 2}"#))
            .finish_reason(FinishReason::ToolCalls);
        let message_3 = Response::new("Task is complete, let me know how can i help you!");

        let tool_result = ToolResult::new(ToolName::new("foo"))
            .content(json!({
                "a": 100,
                "b": 200
            }))
            .use_id(ToolCallId::new("too_call_001"));
        let request = ChatRequest::new("Hello can you help me?");
        let result = Fixture::default()
            .with_provider(
                TestProvider::default()
                    .with_messages(vec![vec![message_1, message_2], vec![message_3.clone()]]),
            )
            .with_tool(tool_result.clone())
            .chat(request)
            .await;

        assert!(result.contains(&ChatResponse::ToolUseStart(ToolUseStart {
            tool_name: Some(ToolName::new("foo"))
        })));
        assert!(result.contains(&ChatResponse::ToolUseEnd(tool_result)));
        assert!(result.contains(&ChatResponse::Text(message_3.content)));
    }

    #[tokio::test]
    async fn test_chat_context() {
        let message_1 = Response::new("Let's use foo tool").add_tool_call(
            ToolCallPart::default()
                .name(ToolName::new("foo"))
                .arguments_part(r#"{"foo": 1,"#)
                .call_id(ToolCallId::new("too_call_001")),
        );
        let message_2 = Response::new("")
            .add_tool_call(ToolCallPart::default().arguments_part(r#""bar": 2}"#))
            .finish_reason(FinishReason::ToolCalls);
        let message_3 = Response::new("Task is complete, let me know how can i help you!");

        let tool_result = ToolResult::new(ToolName::new("foo"))
            .content(json!({
                "a": 100,
                "b": 200
            }))
            .use_id(ToolCallId::new("too_call_001"));
        let request = ChatRequest::new("Hello can you help me?");
        let tester = Fixture::default()
            .with_provider(
                TestProvider::default()
                    .with_messages(vec![vec![message_1, message_2], vec![message_3.clone()]]),
            )
            .with_tool(tool_result.clone());

        let _ = tester.chat(request).await;
        let last_request = tester.provider.get_last_call().unwrap();
        insta::assert_debug_snapshot!(last_request);
    }
}
