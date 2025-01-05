use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ToolCall, ToolCallId, ToolName};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Setters)]
#[setters(strip_option)]
pub struct ToolResult {
    pub name: ToolName,
    pub call_id: Option<ToolCallId>,
    pub content: Value,
    pub is_error: bool,
}

impl ToolResult {
    pub fn new(name: ToolName) -> ToolResult {
        Self {
            name,
            call_id: None,
            content: Value::default(),
            is_error: false,
        }
    }
}

impl From<ToolCall> for ToolResult {
    fn from(value: ToolCall) -> Self {
        Self {
            name: value.name,
            call_id: value.call_id,
            content: Value::default(),
            is_error: false,
        }
    }
}
