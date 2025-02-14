mod agent;
mod chat_request;
mod chat_response;
mod chat_stream_ext;
mod config;
mod context;
mod conversation;
mod env;
mod error;
mod file;
mod learning;
mod message;
mod model;
mod orch;
mod prompt;
mod stream_ext;
mod summarize;
mod tool;
mod tool_call;
mod tool_call_parser;
mod tool_choice;
mod tool_definition;
mod tool_name;
mod tool_result;
mod tool_usage;
mod user_interaction;
mod workflow;

pub use agent::*;
pub use chat_request::*;
pub use chat_response::*;
pub use chat_stream_ext::*;
pub use config::*;
pub use context::*;
pub use conversation::*;
pub use env::*;
pub use error::*;
pub use file::*;
pub use learning::*;
pub use message::*;
pub use model::*;
pub use orch::*;
pub use prompt::*;
pub use stream_ext::*;
pub use summarize::*;
pub use tool::*;
pub use tool_call::*;
pub use tool_call_parser::*;
pub use tool_choice::*;
pub use tool_definition::*;
pub use tool_name::*;
pub use tool_result::*;
pub use tool_usage::*;
pub use user_interaction::*;
pub use workflow::*;

#[async_trait::async_trait]
pub trait ProviderService: Send + Sync + 'static {
    async fn chat(
        &self,
        id: &ModelId,
        context: Context,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error>;
    async fn models(&self) -> anyhow::Result<Vec<Model>>;
    async fn parameters(&self, model: &ModelId) -> anyhow::Result<Parameters>;
}

#[async_trait::async_trait]
pub trait ToolService: Send + Sync {
    // TODO: should take `call` by reference
    async fn call(&self, call: ToolCallFull) -> ToolResult;
    fn list(&self) -> Vec<ToolDefinition>;
    fn usage_prompt(&self) -> String;
}

/// Core app trait providing access to services and repositories.
/// This trait follows clean architecture principles for dependency management
/// and service/repository composition.
pub trait App: Send + Sync + 'static {
    /// The concrete type implementing tool service capabilities
    type ToolService: ToolService;

    /// The concrete type implementing provider service capabilities
    type ProviderService: ProviderService;

    /// Get a reference to the tool service instance
    fn tool_service(&self) -> &Self::ToolService;

    /// Get a reference to the provider service instance
    fn provider_service(&self) -> &Self::ProviderService;
}
