mod chat;
mod completion;
mod env;
mod file_read;
mod root_api;
mod system_prompt;
mod ui;
mod user_prompt;
mod workflow_title;

pub use chat::*;
pub use completion::*;
pub use env::*;
pub use root_api::*;
pub use ui::*;

pub struct Service;

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use anyhow::{bail, Result};
    use derive_setters::Setters;
    use forge_domain::{ChatCompletionMessage, Context, Model, ModelId, Parameters, ResultStream};
    use forge_provider::ProviderService;
    use tokio_stream::StreamExt;

    use super::system_prompt::SystemPromptService;

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
