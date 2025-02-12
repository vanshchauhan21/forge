mod api;
mod executor;
mod suggestion;
mod test;

pub use api::*;
pub use forge_domain::*;
use forge_stream::MpscStream;
pub use test::*;
#[async_trait::async_trait]
pub trait ExecutorService: Send {
    async fn chat(
        &self,
        chat_request: ChatRequest,
    ) -> anyhow::Result<MpscStream<anyhow::Result<AgentMessage<ChatResponse>>>>;

    async fn reset(&self) -> anyhow::Result<()>;
}

#[async_trait::async_trait]
pub trait SuggestionService: Send + Sync {
    async fn suggestions(&self) -> anyhow::Result<Vec<File>>;
}

#[async_trait::async_trait]
pub trait API {
    async fn suggestions(&self) -> anyhow::Result<Vec<File>>;
    async fn tools(&self) -> Vec<ToolDefinition>;
    async fn models(&self) -> anyhow::Result<Vec<Model>>;
    async fn chat(
        &self,
        chat: ChatRequest,
    ) -> anyhow::Result<MpscStream<anyhow::Result<AgentMessage<ChatResponse>, anyhow::Error>>>;
    fn environment(&self) -> Environment;
    async fn reset(&self) -> anyhow::Result<()>;
}
