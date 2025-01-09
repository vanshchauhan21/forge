use std::sync::Arc;

use forge_domain::{
    Config, Context, Environment, Model, ResultStream, ToolDefinition, ToolService,
};
use forge_provider::ProviderService;

use super::chat_service::ConversationHistory;
use super::completion_service::CompletionService;
use super::{
    ChatRequest, ChatResponse, ConfigService, Conversation, ConversationId, ConversationService,
    File, Service, UIService,
};
use crate::{Error, Result};

#[async_trait::async_trait]
pub trait RootAPIService: Send + Sync {
    async fn completions(&self) -> Result<Vec<File>>;
    async fn tools(&self) -> Vec<ToolDefinition>;
    async fn context(&self, conversation_id: ConversationId) -> Result<Context>;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error>;
    async fn conversations(&self) -> Result<Vec<Conversation>>;
    async fn conversation(&self, conversation_id: ConversationId) -> Result<ConversationHistory>;
    async fn get_config(&self) -> Result<Config>;
    async fn set_config(&self, request: Config) -> Result<Config>;
}

impl Service {
    pub fn root_api_service(env: Environment) -> impl RootAPIService {
        Live::new(env)
    }
}

#[derive(Clone)]
struct Live {
    provider: Arc<dyn ProviderService>,
    tool: Arc<dyn ToolService>,
    completions: Arc<dyn CompletionService>,
    ui_service: Arc<dyn UIService>,
    storage: Arc<dyn ConversationService>,
    config_storage: Arc<dyn ConfigService>,
}

impl Live {
    fn new(env: Environment) -> Self {
        let cwd: String = env.cwd.clone();
        let provider = Arc::new(forge_provider::Service::open_router(env.api_key.clone()));
        let tool = Arc::new(forge_tool::Service::tool_service());
        let file_read = Arc::new(Service::file_read_service());

        let system_prompt = Arc::new(Service::system_prompt(
            env.clone(),
            tool.clone(),
            provider.clone(),
        ));
        let user_prompt = Arc::new(Service::user_prompt_service(file_read.clone()));

        let storage =
            Arc::new(Service::storage_service(&cwd).expect("Failed to create storage service"));

        let neo_chat_service = Arc::new(Service::chat_service(
            provider.clone(),
            system_prompt.clone(),
            tool.clone(),
            user_prompt,
        ));
        let completions = Arc::new(Service::completion_service(cwd.clone()));

        let title_service = Arc::new(Service::title_service(provider.clone()));

        let chat_service = Arc::new(Service::ui_service(
            storage.clone(),
            neo_chat_service,
            title_service,
        ));
        let config_storage = Arc::new(
            Service::config_service(&cwd).expect("Failed to create config storage service"),
        );

        Self {
            provider,
            tool,
            completions,
            ui_service: chat_service,
            storage,
            config_storage,
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

    async fn get_config(&self) -> Result<Config> {
        Ok(self.config_storage.get().await?)
    }

    async fn set_config(&self, request: Config) -> Result<Config> {
        self.config_storage.set(request).await
    }
}
