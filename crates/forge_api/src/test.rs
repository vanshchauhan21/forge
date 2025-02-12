use std::sync::Arc;

use anyhow::Result;
use forge_app::{EnvironmentService, ForgeApp, Infrastructure};
use forge_domain::{App, *};
use forge_infra::TestInfra;
use forge_stream::MpscStream;

use crate::executor::ForgeExecutorService;
use crate::suggestion::ForgeSuggestionService;
use crate::{ExecutorService, SuggestionService, API};

pub struct TestAPI<F> {
    app: Arc<F>,
    _executor_service: ForgeExecutorService<F>,
    _suggestion_service: ForgeSuggestionService<F>,
}

impl TestAPI<ForgeApp<TestInfra>> {
    pub fn init(_restricted: bool, large_model_id: ModelId, small_model_id: ModelId) -> Self {
        let infra = Arc::new(TestInfra::new(
            large_model_id.clone(),
            small_model_id.clone(),
        ));
        let app = Arc::new(ForgeApp::new(infra));
        Self {
            app: app.clone(),
            _executor_service: ForgeExecutorService::new(app.clone()),
            _suggestion_service: ForgeSuggestionService::new(app.clone()),
        }
    }
}

#[async_trait::async_trait]
impl<F: App + Infrastructure> API for TestAPI<F> {
    async fn suggestions(&self) -> Result<Vec<File>> {
        self._suggestion_service.suggestions().await
    }

    async fn tools(&self) -> Vec<ToolDefinition> {
        self.app.tool_service().list()
    }

    async fn models(&self) -> Result<Vec<Model>> {
        Ok(self.app.provider_service().models().await?)
    }

    async fn chat(
        &self,
        chat: ChatRequest,
    ) -> anyhow::Result<MpscStream<Result<AgentMessage<ChatResponse>, anyhow::Error>>> {
        Ok(self._executor_service.chat(chat).await?)
    }

    async fn reset(&self) -> anyhow::Result<()> {
        self._executor_service.reset().await
    }

    fn environment(&self) -> Environment {
        self.app.environment_service().get_environment().clone()
    }
}
