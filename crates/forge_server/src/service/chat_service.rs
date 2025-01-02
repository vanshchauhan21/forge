use std::sync::Arc;

use forge_env::Environment;
use forge_provider::{BoxStream, ProviderService, ResultStream, ToolResult};
use forge_tool::ToolService;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use super::Service;
use crate::app::{Action, ChatResponse, Command, FileResponse};
use crate::{Error, Result, SystemPromptService};

#[async_trait::async_trait]
pub trait ChatService: Send + Sync {
    type Command;
    type Action;
    type Error;
    async fn execute(&self, command: &Self::Command) -> ResultStream<Self::Action, Self::Error>;
}

impl Service {
    pub fn chat_service(
        env: Environment,
        api_key: impl Into<String>,
        tx: mpsc::Sender<Result<ChatResponse>>,
    ) -> impl ChatService<Command = Command, Action = Action, Error = Error> {
        Live::new(env, api_key, tx)
    }
}

struct Live {
    provider: Arc<dyn ProviderService>,
    tools: Arc<dyn ToolService>,
    tx: mpsc::Sender<Result<ChatResponse>>,
    system_prompt: Arc<dyn SystemPromptService>,
}

impl Live {
    pub fn new(
        env: Environment,
        api_key: impl Into<String>,
        tx: mpsc::Sender<Result<ChatResponse>>,
    ) -> Self {
        let tool = Arc::new(forge_tool::Service::live());
        let provider = Arc::new(forge_provider::Service::open_router(api_key.into(), None));

        Self {
            provider: provider.clone(),
            tools: tool.clone(),
            tx,
            system_prompt: Arc::new(Service::system_prompt(env, tool, provider)),
        }
    }
}

#[async_trait::async_trait]
impl ChatService for Live {
    type Command = Command;
    type Action = Action;
    type Error = Error;
    async fn execute(&self, command: &Self::Command) -> ResultStream<Self::Action, Self::Error> {
        match command {
            Command::FileRead(files) => {
                let mut responses = vec![];
                for file in files {
                    let content = tokio::fs::read_to_string(file.clone()).await?;
                    responses.push(FileResponse { path: file.to_string(), content });
                }

                let stream: BoxStream<Action, Error> =
                    Box::pin(tokio_stream::once(Ok(Action::FileReadResponse(responses))));

                Ok(stream)
            }
            Command::AssistantMessage(request) => {
                // TODO: To use or not to use tools should be decided by the app and not the
                // executor. Set system prompt based on the model type
                let parameters = self.provider.parameters(&request.model).await?;
                let system_prompt = self.system_prompt.get_system_prompt(&request.model).await?;
                let mut request = request.clone().set_system_message(system_prompt);
                if parameters.tool_supported {
                    request = request.tools(self.tools.list());
                }

                let actions =
                    self.provider.chat(request).await?.map(|response| {
                        response.map(Action::AssistantResponse).map_err(Error::from)
                    });

                Ok(Box::pin(actions))
            }
            Command::UserMessage(message) => {
                match self.tx.send(Ok(message.clone())).await {
                    Ok(_) => {}
                    Err(error) => {
                        error.0?;
                    }
                };

                let stream: BoxStream<Action, Error> = Box::pin(tokio_stream::empty());
                Ok(stream)
            }
            Command::ToolCall(tool_call) => {
                let tool_result = self
                    .tools
                    .call(&tool_call.name, tool_call.arguments.clone())
                    .await;
                let is_error = tool_result.is_err();
                let tool_call_response = Action::ToolResponse(ToolResult {
                    content: match tool_result {
                        Ok(content) => content,
                        Err(e) => serde_json::Value::from(e),
                    },
                    use_id: tool_call.call_id.clone(),
                    name: tool_call.name.clone(),
                    is_error,
                });

                let stream: BoxStream<Action, Error> =
                    Box::pin(tokio_stream::once(Ok(tool_call_response)));

                Ok(stream)
            }
        }
    }
}
