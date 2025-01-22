use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use forge_domain::{
    ChatRequest, ChatResponse, Config, Context, Conversation, ConversationId, Environment, Model,
    ProviderService, ResultStream, ToolDefinition, ToolService,
};

use super::chat::ConversationHistory;
use super::completion::CompletionService;
use super::env::EnvironmentService;
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
        Live::new(std::env::current_dir()?).await
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
    async fn new(cwd: PathBuf) -> Result<Self> {
        let env = Service::environment_service(cwd).get().await?;

        let cwd: String = env.cwd.clone();
        let provider = Arc::new(Service::provider_service(env.api_key.clone()));
        let tool = Arc::new(Service::tool_service());
        let file_read = Arc::new(Service::file_read_service());

        let system_prompt = Arc::new(Service::system_prompt(
            env.clone(),
            tool.clone(),
            provider.clone(),
            file_read.clone(),
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
    use std::path::Path;

    use forge_domain::ModelId;
    use tokio_stream::StreamExt;

    use super::*;

    const MAX_RETRIES: usize = 3;
    const SUPPORTED_MODELS: &[&str] = &[
        "anthropic/claude-3.5-sonnet:beta",
        "openai/gpt-4o-2024-11-20",
        "anthropic/claude-3.5-sonnet",
        "openai/gpt-4o",
        "openai/gpt-4o-mini",
        "google/gemini-flash-1.5",
        "anthropic/claude-3-sonnet",
    ];

    // Helper function for testing model responses
    async fn test_model_responses<T>(
        api: &Live,
        task: String,
        check_response: impl Fn(&str) -> Result<T, String> + Send + Sync + Copy + 'static,
    ) -> Vec<String>
    where
        T: Send + 'static,
    {
        let mut errors = Vec::new();

        for &model in SUPPORTED_MODELS {
            let request = ChatRequest::new(ModelId::new(model), task.clone());

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

                match check_response(&response) {
                    Ok(_) => {
                        println!("[{}] Successfully checked response", model);
                        break;
                    }
                    Err(err) => {
                        if attempt < MAX_RETRIES - 1 {
                            println!(
                                "[{}] Attempt {}/{}: {}",
                                model,
                                attempt + 1,
                                MAX_RETRIES,
                                err
                            );
                        } else {
                            errors.push(format!(
                                "[{}] Failed: {} after {} attempts",
                                model, err, MAX_RETRIES
                            ));
                        }
                    }
                }
            }
        }

        errors
    }

    #[tokio::test]
    async fn test_find_cat_name() {
        let api = Live::new(Path::new("../../").to_path_buf()).await.unwrap();
        let task = "There is a cat hidden in the codebase. What is its name?".to_string();

        let errors = test_model_responses(&api, task, move |response| {
            let response_lower = response.to_lowercase();
            if !response_lower.contains("cat") {
                return Err("Response doesn't mention the cat".to_string());
            }
            if !response_lower.contains("juniper") {
                return Err("Cat's name 'Juniper' not found in response".to_string());
            }
            if !response_lower.contains("code-forge") && !response_lower.contains("codeforge") {
                return Err("Response doesn't mention Code-Forge context".to_string());
            }
            Ok(())
        })
        .await;

        if !errors.is_empty() {
            panic!("Test failures:\n{}", errors.join("\n"));
        }
    }
}
