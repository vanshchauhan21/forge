mod chat_service;
mod completion_service;
mod config_service;
mod conversation_service;
mod db_service;
mod env_service;
mod file_read_service;
mod root_api_service;
mod system_prompt_service;
mod ui_service;
mod user_prompt_service;
mod workflow_title_service;

pub use chat_service::*;
pub use completion_service::*;
pub use config_service::*;
pub use conversation_service::*;
pub use env_service::*;
pub use root_api_service::*;
pub use ui_service::*;

pub struct Service;

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use derive_setters::Setters;
    use forge_domain::{ChatCompletionMessage, Context, Model, ModelId, Parameters, ResultStream};
    use forge_provider::ProviderService;
    use serde_json::json;
    use tokio_stream::StreamExt;

    use super::system_prompt_service::SystemPromptService;
    use crate::Result;

    pub struct TestSystemPrompt {
        prompt: String,
    }

    impl TestSystemPrompt {
        pub fn new(s: impl ToString) -> Self {
            Self { prompt: s.to_string() }
        }
    }

    #[async_trait::async_trait]
    impl SystemPromptService for TestSystemPrompt {
        async fn get_system_prompt(&self, _: &ModelId) -> Result<String> {
            Ok(self.prompt.to_string())
        }
    }

    #[derive(Default, Setters)]
    pub struct TestProvider {
        messages: Mutex<Vec<Vec<ChatCompletionMessage>>>,
        calls: Mutex<Vec<Context>>,
        models: Vec<Model>,
        parameters: Vec<(ModelId, Parameters)>,
    }

    impl TestProvider {
        pub fn with_messages(self, messages: Vec<Vec<ChatCompletionMessage>>) -> Self {
            self.messages(Mutex::new(messages))
        }

        pub fn get_calls(&self) -> Vec<Context> {
            self.calls.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl ProviderService for TestProvider {
        async fn chat(
            &self,
            request: Context,
        ) -> ResultStream<ChatCompletionMessage, forge_provider::Error> {
            self.calls.lock().unwrap().push(request);
            let mut guard = self.messages.lock().unwrap();
            if guard.is_empty() {
                Ok(Box::pin(tokio_stream::empty()))
            } else {
                let response = guard.remove(0);
                Ok(Box::pin(tokio_stream::iter(response).map(Ok)))
            }
        }

        async fn models(&self) -> forge_provider::Result<Vec<Model>> {
            Ok(self.models.clone())
        }

        async fn parameters(&self, model: &ModelId) -> forge_provider::Result<Parameters> {
            match self.parameters.iter().find(|(id, _)| id == model) {
                None => Err(forge_provider::Error::Upstream(
                    json!({"error": "Model not found"}),
                )),
                Some((_, parameter)) => Ok(parameter.clone()),
            }
        }
    }
}
