use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

use super::ToolCallPart;

/// Represents a message that was received from the LLM provider
/// NOTE: ToolUse messages are part of the larger Response object and not part
/// of the message.
#[derive(Default, Clone, Debug, Setters)]
#[setters(into, strip_option)]
pub struct Response {
    pub content: Option<String>,
    pub tool_call: Vec<ToolCallPart>,
    pub finish_reason: Option<FinishReason>,
}

/// The reason why the model stopped generating output.
/// Read more: https://platform.openai.com/docs/guides/function-calling#edge-cases
#[derive(Clone, Debug, Deserialize, Serialize, EnumString, PartialEq, Eq)]
pub enum FinishReason {
    /// The model stopped generating output because it reached the maximum
    /// allowed length.
    #[strum(serialize = "length")]
    Length,
    /// The model stopped generating output because it encountered content that
    /// violated filters.
    #[strum(serialize = "content_filter")]
    ContentFilter,
    /// The model stopped generating output because it made a tool call.
    #[strum(serialize = "tool_calls")]
    ToolCalls,
    /// The model stopped generating output normally.
    #[strum(serialize = "stop", serialize = "end_turn")]
    Stop,
}

impl Response {
    pub fn assistant(content: impl ToString) -> Response {
        Response::default().content(content.to_string())
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_finish_reason_from_str() {
        assert_eq!(
            FinishReason::from_str("length").unwrap(),
            FinishReason::Length
        );
        assert_eq!(
            FinishReason::from_str("content_filter").unwrap(),
            FinishReason::ContentFilter
        );
        assert_eq!(
            FinishReason::from_str("tool_calls").unwrap(),
            FinishReason::ToolCalls
        );
        assert_eq!(FinishReason::from_str("stop").unwrap(), FinishReason::Stop);
        assert_eq!(
            FinishReason::from_str("end_turn").unwrap(),
            FinishReason::Stop
        );
    }
}
