use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use super::ToolCallPart;

/// Represents a message that was received from the LLM provider
/// NOTE: ToolUse messages are part of the larger Response object and not part
/// of the message.
#[derive(Clone, Debug, Setters)]
#[setters(into, strip_option)]
pub struct Response {
    pub content: String,
    pub tool_call: Vec<ToolCallPart>,
    pub finish_reason: Option<FinishReason>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FinishReason {
    ToolCall,
    EndTurn,
}

impl FinishReason {
    pub fn parse(reason: String) -> Option<Self> {
        match reason.as_str() {
            "tool_use" => Some(FinishReason::ToolCall),
            "tool_calls" => Some(FinishReason::ToolCall),
            "end_turn" => Some(FinishReason::EndTurn),
            _ => None,
        }
    }
}

impl Response {
    pub fn assistant(content: impl ToString) -> Response {
        Response::new(content)
    }

    pub fn new(content: impl ToString) -> Response {
        Response {
            content: content.to_string(),
            tool_call: vec![],
            finish_reason: None,
        }
    }

    pub fn add_tool_call(mut self, call_tool: impl Into<ToolCallPart>) -> Self {
        self.tool_call.push(call_tool.into());
        self
    }

    pub fn extend_calls(mut self, calls: Vec<impl Into<ToolCallPart>>) -> Self {
        self.tool_call.extend(calls.into_iter().map(Into::into));
        self
    }

    pub fn finish_reason_opt(mut self, reason: Option<FinishReason>) -> Self {
        self.finish_reason = reason;
        self
    }
}
