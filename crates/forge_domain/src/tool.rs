use schemars::JsonSchema;
use serde_json::Value;

use crate::{ToolCallService, ToolDefinition, ToolDescription};

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
    T::Output: serde::Serialize + JsonSchema,
{
    type Input = Value;
    type Output = Value;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let input: T::Input = serde_json::from_value(input).map_err(|e| e.to_string())?;
        let output: T::Output = self.0.call(input).await?;
        Ok(serde_json::to_value(output).map_err(|e| e.to_string())?)
    }
}

pub struct Tool {
    pub executable: Box<dyn ToolCallService<Input = Value, Output = Value> + Send + Sync + 'static>,
    pub definition: ToolDefinition,
}

impl Tool {
    pub fn new<T>(tool: T) -> Tool
    where
        T: ToolCallService + ToolDescription + Send + Sync + 'static,
        T::Input: serde::de::DeserializeOwned + JsonSchema,
        T::Output: serde::Serialize + JsonSchema,
    {
        let definition = ToolDefinition::new(&tool);
        let executable = Box::new(JsonTool::new(tool));

        Tool { executable, definition }
    }
}
