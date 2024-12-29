use std::sync::Arc;

use forge_env::Environment;
use forge_provider::{BoxStream, Provider, Request, Response, ResultStream, ToolResult};
use forge_tool::ToolEngine;
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

use crate::app::{Action, ChatResponse, Command, FileResponse};
use crate::runtime::Executor;
use crate::Error;

pub struct ChatCommandExecutor {
    provider: Arc<Provider<Request, Response, forge_provider::Error>>,
    tools: Arc<ToolEngine>,
    tx: mpsc::Sender<ChatResponse>,
}

impl ChatCommandExecutor {
    pub fn new(
        env: Environment,
        api_key: impl Into<String>,
        tx: mpsc::Sender<ChatResponse>,
    ) -> Self {
        let tools = ToolEngine::new(env);

        Self {
            provider: Arc::new(Provider::open_router(api_key.into(), None)),
            tools: Arc::new(tools),
            tx,
        }
    }
}

#[async_trait::async_trait]
impl Executor for ChatCommandExecutor {
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
            Command::AssistantMessage(a) => {
                let actions =
                    self.provider.chat(a.clone()).await?.map(|response| {
                        response.map(Action::AssistantResponse).map_err(Error::from)
                    });

                let msg: BoxStream<Action, Error> = Box::pin(actions);

                Ok(msg)
            }
            Command::UserMessage(message) => {
                self.tx.send(message.clone()).await?;

                let stream: BoxStream<Action, Error> = Box::pin(tokio_stream::empty());
                Ok(stream)
            }
            Command::ToolUse(tool_call) => {
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
