use std::sync::Arc;

use forge_env::Environment;
use forge_provider::{Model, ModelId, Provider, Request, ResultStream};
use forge_tool::{ToolDefinition, ToolEngine};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use super::{CompletionService, Service};
use crate::app::{Action, App, ChatRequest, ChatResponse};
use crate::executor::ChatCommandExecutor;
use crate::runtime::ApplicationRuntime;
use crate::{Error, File, Result};

#[async_trait::async_trait]
pub trait APIService: Send + Sync {
    async fn completions(&self) -> Result<Vec<File>>;
    async fn tools(&self) -> Vec<ToolDefinition>;
    async fn context(&self) -> Request;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error>;
}

impl Service {
    pub fn api_service(env: Environment, api_key: impl Into<String>) -> impl APIService {
        Live::new(env, api_key)
    }
}

#[derive(Clone)]
struct Live {
    provider: Arc<Provider>,
    tools: Arc<ToolEngine>,
    completions: Arc<dyn CompletionService>,
    runtime: Arc<ApplicationRuntime<App>>,
    env: Environment,
    api_key: String,
}

impl Live {
    fn new(env: Environment, api_key: impl Into<String>) -> Self {
        let tools = ToolEngine::new();

        let request = Request::new(ModelId::default());

        let cwd: String = env.cwd.clone();
        let api_key: String = api_key.into();

        Self {
            env,
            provider: Arc::new(Provider::open_router(api_key.clone(), None)),
            tools: Arc::new(tools),
            completions: Arc::new(Service::completion_service(cwd.clone())),
            runtime: Arc::new(ApplicationRuntime::new(App::new(request))),
            api_key,
        }
    }
}

#[async_trait::async_trait]
impl APIService for Live {
    async fn completions(&self) -> Result<Vec<File>> {
        self.completions.list().await
    }

    async fn tools(&self) -> Vec<ToolDefinition> {
        self.tools.list()
    }

    async fn context(&self) -> Request {
        self.runtime.state().await.request().clone()
    }

    async fn models(&self) -> Result<Vec<Model>> {
        Ok(self.provider.models().await?)
    }

    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error> {
        let (tx, rx) = mpsc::channel::<Result<ChatResponse>>(100);
        let executor = ChatCommandExecutor::new(self.env.clone(), self.api_key.clone(), tx);
        let runtime = self.runtime.clone();
        let message = format!("<task>{}</task>", chat.content);

        tokio::spawn(async move {
            runtime
                .clone()
                .execute(
                    Action::UserMessage(chat.content(message)),
                    Arc::new(executor),
                )
                .await
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}
