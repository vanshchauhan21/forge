use std::sync::Arc;

use anyhow::Result;
use forge_domain::{
    ChatRequest, ChatResponse, Config, Context, Conversation, ConversationId, Environment, Model,
    ProviderService, ResultStream, ToolDefinition,
};

use super::chat::ConversationHistory;
use super::completion::CompletionService;
use super::env::EnvironmentService;
use super::tool_service::ToolService;
use super::{File, Service, UIService};
use crate::{ConfigRepository, ConversationRepository};

#[async_trait::async_trait]
pub trait APIService: Send + Sync {
    async fn completions(&self) -> Result<Vec<File>>;
    async fn tools(&self) -> Vec<ToolDefinition>;
    async fn context(&self, conversation_id: ConversationId) -> Result<Context>;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, anyhow::Error>;
    async fn conversations(&self) -> Result<Vec<Conversation>>;
    async fn conversation(&self, conversation_id: ConversationId) -> Result<ConversationHistory>;
    async fn get_config(&self) -> Result<Config>;
    async fn set_config(&self, request: Config) -> Result<Config>;
    async fn environment(&self) -> Result<Environment>;
}

impl Service {
    pub async fn api_service() -> Result<impl APIService> {
        Live::new().await
    }
}

#[derive(Clone)]
struct Live {
    provider: Arc<dyn ProviderService>,
    tool: Arc<dyn ToolService>,
    completions: Arc<dyn CompletionService>,
    ui_service: Arc<dyn UIService>,
    storage: Arc<dyn ConversationRepository>,
    config_storage: Arc<dyn ConfigRepository>,
    environment: Environment,
}

impl Live {
    async fn new() -> Result<Self> {
        let env = Service::environment_service().get().await?;

        let cwd: String = env.cwd.clone();
        let provider = Arc::new(Service::provider_service(env.api_key.clone()));
        let tool = Arc::new(Service::tool_service());
        let file_read = Arc::new(Service::file_read_service());

        let system_prompt = Arc::new(Service::system_prompt(
            env.clone(),
            tool.clone(),
            provider.clone(),
        ));

        let user_prompt = Arc::new(Service::user_prompt_service(file_read.clone()));
        let storage = Arc::new(Service::storage_service(&cwd)?);

        let chat_service = Arc::new(Service::chat_service(
            provider.clone(),
            system_prompt.clone(),
            tool.clone(),
            user_prompt,
        ));
        let completions = Arc::new(Service::completion_service(cwd.clone()));

        let title_service = Arc::new(Service::title_service(provider.clone()));

        let chat_service = Arc::new(Service::ui_service(
            storage.clone(),
            chat_service,
            title_service,
        ));
        let config_storage = Arc::new(Service::config_service(&cwd)?);

        Ok(Self {
            provider,
            tool,
            completions,
            ui_service: chat_service,
            storage,
            config_storage,
            environment: env,
        })
    }
}

#[async_trait::async_trait]
impl APIService for Live {
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

    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, anyhow::Error> {
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

    async fn environment(&self) -> Result<Environment> {
        Ok(self.environment.clone())
    }
}

#[cfg(test)]
mod tests {
    use forge_domain::ModelId;
    use tokio_stream::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test_e2e() {
        const MAX_RETRIES: usize = 3;
        const MATCH_THRESHOLD: f64 = 0.7; // 70% of crates must be found

        let api = Live::new().await.unwrap();
        let task = include_str!("./api_task.md");
        let request = ChatRequest::new(ModelId::new("anthropic/claude-3.5-sonnet"), task);

        let expected_crates = [
            "forge_app",
            "forge_ci",
            "forge_domain",
            "forge_main",
            "forge_open_router",
            "forge_prompt",
            "forge_tool",
            "forge_tool_macros",
            "forge_walker",
        ];

        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            let response = api
                .chat(request.clone())
                .await
                .unwrap()
                .filter_map(|message| match message.unwrap() {
                    ChatResponse::Text(text) => Some(text),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .await
                .join("")
                .trim()
                .to_string();

            let found_crates: Vec<&str> = expected_crates
                .iter()
                .filter(|&crate_name| response.contains(&format!("<crate>{}</crate>", crate_name)))
                .cloned()
                .collect();

            let match_percentage = found_crates.len() as f64 / expected_crates.len() as f64;

            if match_percentage >= MATCH_THRESHOLD {
                println!(
                    "Successfully found {:.2}% of expected crates",
                    match_percentage * 100.0
                );
                return;
            }

            last_error = Some(format!(
                "Attempt {}: Only found {}/{} crates: {:?}",
                attempt + 1,
                found_crates.len(),
                expected_crates.len(),
                found_crates
            ));

            // Add a small delay between retries to allow for different LLM generations
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }

        panic!(
            "Failed after {} attempts. Last error: {}",
            MAX_RETRIES,
            last_error.unwrap_or_default()
        );
    }
}
