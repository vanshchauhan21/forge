mod api;
mod chat;
mod completion;
mod env;
mod file_read;
mod provider;
mod system_prompt;
#[cfg(test)]
mod test;
mod tool_service;
mod ui;
mod user_prompt;
mod workflow_title;

pub use api::*;
pub use completion::*;
use forge_domain::ChatRequest;
pub use ui::*;

pub struct Service;

#[async_trait::async_trait]
pub trait PromptService: Send + Sync {
    /// Generate prompt from a ChatRequest
    async fn get(&self, request: &ChatRequest) -> anyhow::Result<String>;
}
