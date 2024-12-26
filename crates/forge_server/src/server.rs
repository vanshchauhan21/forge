use std::sync::Arc;

use forge_provider::{Message, Model, ModelId, Provider, Request, Response};
use forge_tool::{Tool, ToolEngine};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::Stream;

use crate::app::{Action, App, ChatRequest, ChatResponse};
use crate::completion::{Completion, File};
use crate::executor::ChatCommandExecutor;
use crate::runtime::ApplicationRuntime;
use crate::Result;

#[derive(Clone)]
pub struct Server {
    provider: Arc<Provider<Request, Response, forge_provider::Error>>,
    tools: Arc<ToolEngine>,
    completions: Arc<Completion>,
    runtime: Arc<ApplicationRuntime<App>>,

    api_key: String,
}

impl Server {
    pub fn new(cwd: impl Into<String>, api_key: impl Into<String>) -> Server {
        let tools = ToolEngine::default();
        let request = Request::new(ModelId::default())
            .add_message(Message::system(include_str!("./prompts/system.md")))
            .tools(tools.list());

        let cwd: String = cwd.into();
        let api_key: String = api_key.into();
        Self {
            provider: Arc::new(Provider::open_router(api_key.clone(), None)),
            tools: Arc::new(tools),
            completions: Arc::new(Completion::new(cwd.clone())),
            runtime: Arc::new(ApplicationRuntime::new(App::new(request))),

            api_key,
        }
    }

    pub async fn completions(&self) -> Result<Vec<File>> {
        self.completions.list().await
    }

    pub fn tools(&self) -> Vec<Tool> {
        self.tools.list()
    }

    pub async fn context(&self) -> App {
        self.runtime.state().await
    }

    pub async fn models(&self) -> Result<Vec<Model>> {
        Ok(self.provider.models().await?)
    }

    pub async fn chat(&self, chat: ChatRequest) -> Result<impl Stream<Item = ChatResponse> + Send> {
        let (tx, rx) = mpsc::channel::<ChatResponse>(100);
        let files = self.completions.list().await?;
        let file_message = files
            .into_iter()
            .map(|file| file.path)
            .fold("".to_string(), |acc, file| format!("{}{}", acc, file));
        let executor = ChatCommandExecutor::new(tx, self.api_key.clone());

        let runtime = self.runtime.clone();
        let message = format!(
            "##Requirements\n{}\n##nCurrent Files\n{}",
            chat.message, file_message
        );

        tokio::spawn(async move {
            runtime
                .clone()
                .execute(Action::UserChatMessage(chat.message(message)), &executor)
                .await
        });

        Ok(ReceiverStream::new(rx))
    }
}
