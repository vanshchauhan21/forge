use schemars::JsonSchema;
use serde_json::Value;

use crate::{NamedTool, ToolCallService, ToolDefinition, ToolDescription};

struct JsonTool<T>(T);

impl<T> JsonTool<T> {
    pub fn new(tool: T) -> Self {
        Self(tool)
    }
}

#[async_trait::async_trait]
impl<T: ToolCallService + Sync> ToolCallService for JsonTool<T>
where
    T::Input: serde::de::DeserializeOwned + JsonSchema,
{
    type Input = Value;

    async fn call(&self, input: Self::Input) -> Result<String, String> {
        let input: T::Input = serde_json::from_value(input).map_err(|e| e.to_string())?;
        self.0.call(input).await
    }
}

pub struct Tool {
    pub executable: Box<dyn ToolCallService<Input = Value> + Send + Sync + 'static>,
    pub definition: ToolDefinition,
}

impl<T> From<T> for Tool
where
    T: ToolCallService + ToolDescription + NamedTool + Send + Sync + 'static,
    T::Input: serde::de::DeserializeOwned + JsonSchema,
{
    fn from(tool: T) -> Self {
        let definition = ToolDefinition::from(&tool);
        let executable = Box::new(JsonTool::new(tool));

        Tool { executable, definition }
    }
}
