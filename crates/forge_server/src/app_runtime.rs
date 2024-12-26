use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use forge_provider::{BoxStream, Message, ModelId, Provider, Request, Response, ResultStream};
use forge_tool::ToolEngine;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};

use crate::app::{Action, App, ChatRequest, ChatResponse, Command, FileResponse};
use crate::completion::Completion;
use crate::Error;

pub struct AppRuntime {
    provider: Arc<Provider<Request, Response, forge_provider::Error>>,
    tools: Arc<ToolEngine>,
    context: Arc<Mutex<App>>,
    completions: Arc<Completion>,
}

impl AppRuntime {
    pub fn new(cwd: impl Into<String>, api_key: impl Into<String>) -> Self {
        let tools = ToolEngine::default();
        let context = Request::new(ModelId::default())
            .add_message(Message::system(include_str!("./prompts/system.md")))
            .tools(tools.list());

        Self {
            provider: Arc::new(Provider::open_router(api_key.into(), None)),
            tools: Arc::new(tools),
            context: Arc::new(Mutex::new(App::new(context))),
            completions: Arc::new(Completion::new(cwd)),
        }
    }

    pub fn chat(
        &self,
        chat: ChatRequest,
    ) -> crate::Result<impl Stream<Item = ChatResponse> + Send> {
        let (tx, rx) = mpsc::channel::<ChatResponse>(100);
        let app = self.context.lock().unwrap().clone();
        let (app, command) = app.run(Action::UserChatMessage(chat))?;

        match command {
            Command::Empty => {}
            Command::Combine(a, b) => {}
            Command::LoadPromptFiles(_) => {}
            Command::DispatchAgentMessage(_) => {}
            Command::DispatchUserMessage(_) => {}
            Command::DispatchToolUse(_, _) => {}
        }

        Ok(ReceiverStream::new(rx))
    }

    #[async_recursion::async_recursion]
    async fn command_executor(
        &self,
        command: &Command,
        tx: &Sender<ChatResponse>,
    ) -> ResultStream<Action, Error> {
        match command {
            Command::Empty => {
                let stream: BoxStream<Action, Error> = Box::pin(tokio_stream::empty());
                Ok(stream)
            }
            Command::Combine(a, b) => {
                let merged: BoxStream<Action, Error> = Box::pin(
                    self.command_executor(a, tx)
                        .await?
                        .merge(self.command_executor(b, tx).await?),
                );

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
                // let stream = self.provider.chat(a.clone()).await?;

                // let actions = stream.map(|response| response.map(|a|
                // Action::AgentChatResponse(a)));

                // let msg: BoxStream<Action, Error> = Box::pin(actions);

                // Ok(msg)

                todo!()
            }
            Command::DispatchUserMessage(message) => {
                tx.send(ChatResponse::Text(message.clone())).unwrap();

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
