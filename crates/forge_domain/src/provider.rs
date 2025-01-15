use anyhow::Result;

use crate::{
    ChatCompletionMessage, Context as ChatContext, Model, ModelId, Parameters, ResultStream,
};

#[async_trait::async_trait]
pub trait ProviderService: Send + Sync + 'static {
    async fn chat(
        &self,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error>;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn parameters(&self, model: &ModelId) -> Result<Parameters>;
}
