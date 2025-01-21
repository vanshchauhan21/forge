mod api;
mod chat;
mod completion;
mod env;
mod file_read;
mod provider;
mod system_prompt;
mod tool_service;
mod ui;
mod user_prompt;
mod workflow_title;

pub use api::*;
pub use chat::*;
pub use completion::*;
use forge_domain::ChatRequest;
pub use ui::*;

pub struct Service;

#[async_trait::async_trait]
pub trait PromptService: Send + Sync {
    /// Generate prompt from a ChatRequest
    async fn get(&self, request: &ChatRequest) -> anyhow::Result<String>;
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use anyhow::{bail, Result};
    use derive_setters::Setters;
    use forge_domain::{
        ChatCompletionMessage, ChatRequest, Context, Model, ModelId, Parameters, ProviderService,
        ResultStream,
    };
    use tokio_stream::StreamExt;

    use super::PromptService;

    #[derive(Default, Setters)]
    #[setters(into, strip_option)]
    pub struct TestPrompt {
        system: Option<String>,
    }

    #[async_trait::async_trait]
    impl PromptService for TestPrompt {
        async fn get(&self, request: &ChatRequest) -> Result<String> {
            let content = match self.system.clone() {
                None => format!("<task>{}</task>", request.content),
                Some(prompt) => prompt,
            };

            Ok(content)
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
            _model_id: &ModelId,
            request: Context,
        ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
            self.calls.lock().unwrap().push(request);
            let mut guard = self.messages.lock().unwrap();
            if guard.is_empty() {
                Ok(Box::pin(tokio_stream::empty()))
            } else {
                let response = guard.remove(0);
                Ok(Box::pin(tokio_stream::iter(response).map(Ok)))
            }
        }

        async fn models(&self) -> Result<Vec<Model>> {
            Ok(self.models.clone())
        }

        async fn parameters(&self, model: &ModelId) -> Result<Parameters> {
            match self.parameters.iter().find(|(id, _)| id == model) {
                None => bail!("Model not found: {}", model),
                Some((_, parameter)) => Ok(parameter.clone()),
            }
        }
    }
}
