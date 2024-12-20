use std::collections::HashMap;

use model::{CallToolRequest, CallToolResponse, ToolsListResponse};

mod fs;
mod model;
mod think;
use serde_json::Value;

// TODO: use a more type-safe API instead of the MCP interface
#[async_trait::async_trait]
pub(crate) trait ToolTrait {
    fn name(&self) -> &'static str;

    async fn tools_call(&self, input: CallToolRequest) -> Result<CallToolResponse, String>;
    fn tools_list(&self) -> ToolsListResponse;
}

#[derive(Default)]

pub struct ToolEngine {
    tools: HashMap<ToolId, Box<dyn ToolTrait>>,
}

#[derive(Debug, Clone)]
pub struct JsonSchema(Value);

impl JsonSchema {
    pub(crate) fn from_value(value: Value) -> Self {
        JsonSchema(value)
    }

    pub fn into_value(self) -> Value {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub id: ToolId,
    pub description: String,
    pub input_schema: JsonSchema,
    pub output_schema: Option<JsonSchema>,
}

#[derive(Debug, Clone)]
pub struct ToolId(String);

impl ToolId {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl ToolEngine {
    pub async fn call(&self, tool_id: ToolId, input: Value) -> Result<Value, String> {
        todo!()
    }

    pub fn list(&self) -> Vec<Tool> {
        todo!()
    }
}
