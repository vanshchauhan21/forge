use std::sync::Arc;

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
    pub fn new(tx: mpsc::Sender<ChatResponse>, api_key: impl Into<String>) -> Self {
        let tools = ToolEngine::default();

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
            Command::ToolUse(tool_use) => {
                let tool_result = self
                    .tools
                    .call(&tool_use.name, tool_use.arguments.clone())
                    .await;
                let is_error = tool_result.is_err();
                let tool_use_response = Action::ToolResponse(ToolResult {
                    content: match tool_result {
                        Ok(content) => content,
                        Err(e) => serde_json::Value::from(e),
                    },
                    tool_use_id: tool_use.use_id.clone(),
                    tool_name: tool_use.name.clone(),
                    is_error,
                });

                let stream: BoxStream<Action, Error> =
                    Box::pin(tokio_stream::once(Ok(tool_use_response)));

                Ok(stream)
            }
        }
    }
}
