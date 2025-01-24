use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use forge_domain::{
    ChatRequest, ChatResponse, Config, ConfigRepository, Context, Conversation, ConversationId,
    ConversationRepository, Environment, Model, ProviderService, ResultStream, ToolDefinition,
    ToolService,
};

use super::chat::ConversationHistory;
use super::completion::CompletionService;
use super::env::EnvironmentService;
use super::{File, Service, UIService};

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
    conversation_repo: Arc<dyn ConversationRepository>,
    config_repo: Arc<dyn ConfigRepository>,
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

        let sqlite = Arc::new(Service::db_pool_service(&cwd)?);

        let conversation_repo = Arc::new(Service::conversation_repo(sqlite.clone()));

        let config_repo = Arc::new(Service::config_repo(sqlite.clone()));

        let chat_service = Arc::new(Service::chat_service(
            provider.clone(),
            system_prompt.clone(),
            tool.clone(),
            user_prompt,
        ));
        let completions = Arc::new(Service::completion_service(cwd.clone()));

        let title_service = Arc::new(Service::title_service(provider.clone()));

        let chat_service = Arc::new(Service::ui_service(
            conversation_repo.clone(),
            chat_service,
            title_service,
        ));

        Ok(Self {
            provider,
            tool,
            completions,
            ui_service: chat_service,
            conversation_repo,
            config_repo,
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
        Ok(self.conversation_repo.get(conversation_id).await?.context)
    }

    async fn models(&self) -> Result<Vec<Model>> {
        Ok(self.provider.models().await?)
    }

    async fn chat(&self, chat: ChatRequest) -> ResultStream<ChatResponse, anyhow::Error> {
        Ok(self.ui_service.chat(chat).await?)
    }

    async fn conversations(&self) -> Result<Vec<Conversation>> {
        self.conversation_repo.list().await
    }

    async fn conversation(&self, conversation_id: ConversationId) -> Result<ConversationHistory> {
        Ok(self
            .conversation_repo
            .get(conversation_id)
            .await?
            .context
            .into())
    }

    async fn get_config(&self) -> Result<Config> {
        Ok(self.config_repo.get().await?)
    }

    async fn set_config(&self, config: Config) -> Result<Config> {
        self.config_repo.set(config).await
    }

    async fn environment(&self) -> Result<Environment> {
        Ok(self.environment.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use forge_domain::ModelId;
    use futures::future::join_all;
    use tokio_stream::StreamExt;

    use super::*;

    const MAX_RETRIES: usize = 3;
    const SUPPORTED_MODELS: &[&str] = &[
        "anthropic/claude-3.5-sonnet:beta",
        "openai/gpt-4o-2024-11-20",
        "anthropic/claude-3.5-sonnet",
        "openai/gpt-4o",
        "openai/gpt-4o-mini",
        // "google/gemini-flash-1.5",
        "anthropic/claude-3-sonnet",
    ];

    /// Test fixture for API testing that supports parallel model validation
    struct Fixture {
        task: String,
    }

    impl Fixture {
        /// Create a new test fixture with the given task
        fn new(task: impl Into<String>) -> Self {
            Self { task: task.into() }
        }

        /// Get the API service, panicking if not validated
        async fn api(&self) -> Live {
            Live::new(Path::new("../../").to_path_buf()).await.unwrap()
        }

        /// Get model response as text
        async fn get_model_response(&self, model: &str) -> String {
            let request = ChatRequest::new(ModelId::new(model), self.task.clone());
            self.api()
                .await
                .chat(request)
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
                .to_string()
        }

        /// Test single model with retries
        async fn test_single_model(
            &self,
            model: &str,
            check_response: impl Fn(&str) -> bool,
        ) -> Result<(), String> {
            for attempt in 0..MAX_RETRIES {
                let response = self.get_model_response(model).await;

                if check_response(&response) {
                    println!("[{}] Successfully checked response", model);
                    return Ok(());
                }

                if attempt < MAX_RETRIES - 1 {
                    println!("[{}] Attempt {}/{}", model, attempt + 1, MAX_RETRIES);
                }
            }
            Err(format!("[{}] Failed after {} attempts", model, MAX_RETRIES))
        }

        /// Run tests for all models in parallel
        async fn test_models(
            &self,
            check_response: impl Fn(&str) -> bool + Send + Sync + Copy + 'static,
        ) -> Vec<String> {
            let futures = SUPPORTED_MODELS
                .iter()
                .map(|&model| async move { self.test_single_model(model, check_response).await });

            join_all(futures)
                .await
                .into_iter()
                .filter_map(Result::err)
                .collect()
        }
    }

    #[tokio::test]
    async fn test_find_cat_name() -> Result<()> {
        let errors = Fixture::new("There is a cat hidden in the codebase. What is its name?")
            .test_models(|response| response.to_lowercase().contains("juniper"))
            .await;

        assert!(errors.is_empty(), "Test failures:\n{}", errors.join("\n"));
        Ok(())
    }
}
