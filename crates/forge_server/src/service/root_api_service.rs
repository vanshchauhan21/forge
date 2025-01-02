use std::sync::Arc;

use forge_env::Environment;
use forge_provider::{Model, ProviderService, Request, ResultStream};
use forge_tool::{ToolDefinition, ToolService};

use super::neo_chat_service::NeoChatService;
use super::{CompletionService, Service};
use crate::{ChatRequest, ChatResponse, Error, File, Result};

#[async_trait::async_trait]
pub trait RootAPIService: Send + Sync {
    async fn completions(&self) -> Result<Vec<File>>;
    async fn tools(&self) -> Vec<ToolDefinition>;
    async fn context(&self) -> Request;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error>;
}

impl Service {
    pub fn root_api_service(env: Environment, api_key: impl Into<String>) -> impl RootAPIService {
        Live::new(env, api_key)
    }
}

#[derive(Clone)]
struct Live {
    provider: Arc<dyn ProviderService>,
    tool: Arc<dyn ToolService>,
    completions: Arc<dyn CompletionService>,
    chat_service: Arc<dyn NeoChatService>,
}

impl Live {
    fn new(env: Environment, api_key: impl Into<String>) -> Self {
        let cwd: String = env.cwd.clone();
        let api_key: String = api_key.into();
        let provider = Arc::new(forge_provider::Service::open_router(api_key.clone(), None));
        let tool = Arc::new(forge_tool::Service::live());

        let system_prompt = Arc::new(Service::system_prompt(
            env.clone(),
            tool.clone(),
            provider.clone(),
        ));

        let chat_service = Arc::new(Service::neo_chat_service(
            provider.clone(),
            system_prompt.clone(),
            tool.clone(),
        ));

        let completions = Arc::new(Service::completion_service(cwd.clone()));
        Self { provider, tool, completions, chat_service }
    }
}

#[async_trait::async_trait]
impl RootAPIService for Live {
    async fn completions(&self) -> Result<Vec<File>> {
        self.completions.list().await
    }

    async fn tools(&self) -> Vec<ToolDefinition> {
        self.tool.list()
    }

    async fn context(&self) -> Request {
        todo!("Implement it via the storage API");
    }

    async fn models(&self) -> Result<Vec<Model>> {
        Ok(self.provider.models().await?)
    }

    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error> {
        Ok(self.chat_service.chat(chat).await?)
    }
}
