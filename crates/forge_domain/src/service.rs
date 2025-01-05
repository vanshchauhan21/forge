use serde_json::Value;

use crate::{ToolDefinition, ToolName};

#[async_trait::async_trait]
pub trait ToolService: Send + Sync {
    async fn call(&self, name: &ToolName, input: Value) -> Result<Value, String>;
    fn list(&self) -> Vec<ToolDefinition>;
    fn usage_prompt(&self) -> String;
}
