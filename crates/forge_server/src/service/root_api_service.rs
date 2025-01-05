use std::sync::Arc;

use forge_domain::{Context, Model, ResultStream, ToolDefinition, ToolService};
use forge_env::Environment;
use forge_provider::ProviderService;

use super::chat_service::ConversationHistory;
use super::completion_service::CompletionService;
use super::{ConversationId, ConversationService, Service, UIService};
use crate::{ChatRequest, ChatResponse, Conversation, Error, File, Result};

#[async_trait::async_trait]
pub trait RootAPIService: Send + Sync {
    async fn completions(&self) -> Result<Vec<File>>;
    async fn tools(&self) -> Vec<ToolDefinition>;
    async fn context(&self, conversation_id: ConversationId) -> Result<Context>;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error>;
    async fn conversations(&self) -> Result<Vec<Conversation>>;
    async fn conversation(&self, conversation_id: ConversationId) -> Result<ConversationHistory>;
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
    ui_service: Arc<dyn UIService>,
    storage: Arc<dyn ConversationService>,
}

impl Live {
    fn new(env: Environment, api_key: impl Into<String>) -> Self {
        let cwd: String = env.cwd.clone();
        let api_key: String = api_key.into();
        let provider = Arc::new(forge_provider::Service::open_router(api_key));
        let tool = Arc::new(forge_tool::Service::tool_service());

        let system_prompt = Arc::new(Service::system_prompt(
            env.clone(),
            tool.clone(),
            provider.clone(),
        ));
        let file_read = Arc::new(Service::file_read_service());
        let user_prompt = Arc::new(Service::user_prompt_service(file_read));

        let storage =
            Arc::new(Service::storage_service(&cwd).expect("Failed to create storage service"));

        let neo_chat_service = Arc::new(Service::chat_service(
            provider.clone(),
            system_prompt.clone(),
            tool.clone(),
            user_prompt,
        ));

        let chat_service = Arc::new(Service::ui_service(storage.clone(), neo_chat_service));

        let completions = Arc::new(Service::completion_service(cwd.clone()));

        let storage =
            Arc::new(Service::storage_service(&cwd).expect("Failed to create storage service"));

        Self {
            provider,
            tool,
            completions,
            ui_service: chat_service,
            storage,
        }
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

    async fn context(&self, conversation_id: ConversationId) -> Result<Context> {
        Ok(self
            .storage
            .get_conversation(conversation_id)
            .await?
            .context)
    }

    async fn models(&self) -> Result<Vec<Model>> {
        Ok(self.provider.models().await?)
    }

    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error> {
        Ok(self.ui_service.chat(chat).await?)
    }

    async fn conversations(&self) -> Result<Vec<Conversation>> {
        self.storage.list_conversations().await
    }

    async fn conversation(&self, conversation_id: ConversationId) -> Result<ConversationHistory> {
        Ok(self
            .storage
            .get_conversation(conversation_id)
            .await?
            .context
            .into())
    }
}
