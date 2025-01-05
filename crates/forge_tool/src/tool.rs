use forge_domain::{Description, ToolCallService, ToolDefinition};
use schemars::JsonSchema;
use serde_json::Value;

use crate::tool_call_service::JsonTool;

pub struct Tool {
    pub executable: Box<dyn ToolCallService<Input = Value, Output = Value> + Send + Sync + 'static>,
    pub definition: ToolDefinition,
}

impl Tool {
    pub fn new<T>(tool: T) -> Tool
    where
        T: ToolCallService + Description + Send + Sync + 'static,
        T::Input: serde::de::DeserializeOwned + JsonSchema,
        T::Output: serde::Serialize + JsonSchema,
    {
        let executable = Box::new(JsonTool::new(tool));
        let description = T::description().to_string();
        let definition = ToolDefinition::new::<T>(description);

        Tool { executable, definition }
    }
}
