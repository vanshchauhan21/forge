use serde_json::Value;

use crate::{Error, Model, ModelId, Parameters, Request, Response, Result, ResultStream, ToolDefinition, ToolName};

#[async_trait::async_trait]
pub trait ToolService: Send + Sync {
    async fn call(&self, name: &ToolName, input: Value) -> std::result::Result<Value, String>;
    fn list(&self) -> Vec<ToolDefinition>;
    fn usage_prompt(&self) -> String;
}

#[async_trait::async_trait]
pub trait ProviderService: Send + Sync + 'static {
    async fn chat(&self, request: Request) -> ResultStream<Response, Error>;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn parameters(&self, model: &ModelId) -> Result<Parameters>;
}
