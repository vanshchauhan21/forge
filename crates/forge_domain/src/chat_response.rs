use serde::Serialize;
use serde_json::Value;

use crate::{ToolCallFull, ToolResult, Usage};

/// Events that are emitted by the agent for external consumption. This includes
/// events for all internal state changes.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ChatResponse {
    Text(String),
    ToolCallStart(ToolCallFull),
    ToolCallEnd(ToolResult),
    Usage(Usage),
    VariableSet { key: String, value: Value },
}
