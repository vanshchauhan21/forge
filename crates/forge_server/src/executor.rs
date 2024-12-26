use std::sync::Arc;

use forge_provider::{BoxStream, Provider, Request, Response, ResultStream};
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
        dbg!(command);
        match command {
            Command::Empty => {
                let stream: BoxStream<Action, Error> = Box::pin(tokio_stream::empty());
                Ok(stream)
            }
            Command::Combine(a, b) => {
                let merged: BoxStream<Action, Error> =
                    Box::pin(self.execute(a).await?.merge(self.execute(b).await?));

                Ok(merged)
            }
            Command::LoadPromptFiles(files) => {
                let mut responses = vec![];
                for file in files {
                    let content = tokio::fs::read_to_string(file.clone()).await?;
                    responses.push(FileResponse { path: file.to_string(), content });
                }

                let stream: BoxStream<Action, Error> =
                    Box::pin(tokio_stream::once(Ok(Action::PromptFileLoaded(responses))));

                Ok(stream)
            }
            Command::DispatchAgentMessage(a) => {
                let actions =
                    self.provider.chat(a.clone()).await?.map(|response| {
                        response.map(Action::AgentChatResponse).map_err(Error::from)
                    });

                let msg: BoxStream<Action, Error> = Box::pin(actions);

                Ok(msg)
            }
            Command::DispatchUserMessage(message) => {
                self.tx.send(ChatResponse::Text(message.clone())).await?;

                let stream: BoxStream<Action, Error> = Box::pin(tokio_stream::empty());
                Ok(stream)
            }
            Command::DispatchToolUse(a, b) => {
                let tool_use_response = Action::ToolUseResponse(serde_json::to_string(
                    &self.tools.call(a, b.clone()).await?,
                )?);

                let stream: BoxStream<Action, Error> =
                    Box::pin(tokio_stream::once(Ok(tool_use_response)));

                Ok(stream)
            }
        }
    }
}
