use futures::StreamExt;

use super::buffered_stream::{scan_stream, Collect};
use super::{ChatCompletionMessage, FinishReason, ToolCall, ToolCallFull, ToolCallPart};
use crate::{BoxStream, Error};

pub trait BoxStreamExt {
    /// Collects all the tool parts to create a full tool call
    fn with_tool_calls(self) -> Self;
}

impl BoxStreamExt for BoxStream<ChatCompletionMessage, Error> {
    fn with_tool_calls(self) -> Self {
        scan_stream(
            self,
            Vec::<ToolCallPart>::new(),
            |parts, message| match message {
                Ok(ChatCompletionMessage { tool_call, finish_reason, .. }) => {
                    if let Some(ToolCall::Part(tool_call)) = tool_call.first() {
                        parts.push(tool_call.clone());
                    }

                    if let Some(FinishReason::ToolCalls) = finish_reason {
                        Collect::Ready
                    } else {
                        Collect::Continue
                    }
                }
                _ => Collect::Continue,
            },
            |parts| {
                let tool_call = ToolCallFull::try_from_parts(parts)?;
                Ok(ChatCompletionMessage::default().add_tool_call(tool_call.clone()))
            },
        )
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use futures::stream;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::ToolName;

    /// Tests that tool call parts are properly collected and combined into a
    /// full tool call when receiving valid tool call parts with a finish
    /// reason
    #[tokio::test]
    async fn test_with_tool_calls_basic() {
        // Create a stream of messages with tool call parts
        let messages = vec![
            ChatCompletionMessage::default().add_tool_call(
                ToolCallPart::default()
                    .name(ToolName::new("test_tool"))
                    .arguments_part("{\"key\":"),
            ),
            ChatCompletionMessage::default()
                .add_tool_call(ToolCallPart::default().arguments_part("\"value\"}"))
                .finish_reason_opt(Some(FinishReason::ToolCalls)),
        ];

        let actual = stream::iter(messages.into_iter().map(Ok))
            .boxed()
            .with_tool_calls()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .await;

        let expected = vec![
            ChatCompletionMessage::default().add_tool_call(
                ToolCallPart::default()
                    .name(ToolName::new("test_tool"))
                    .arguments_part("{\"key\":"),
            ),
            ChatCompletionMessage::default()
                .add_tool_call(ToolCallPart::default().arguments_part("\"value\"}"))
                .finish_reason_opt(Some(FinishReason::ToolCalls)),
            ChatCompletionMessage::default().add_tool_call(
                ToolCallFull::new(ToolName::new("test_tool")).arguments(json!({"key": "value"})),
            ),
        ];

        assert_eq!(actual, expected);
    }

    /// Tests that an empty stream of messages produces an empty result
    #[tokio::test]
    async fn test_with_tool_calls_empty_stream() {
        let messages: Vec<Result<ChatCompletionMessage, Error>> = vec![];

        let actual = stream::iter(messages)
            .boxed()
            .with_tool_calls()
            .collect::<Vec<_>>()
            .await;

        assert!(actual.is_empty());
    }

    /// Tests that messages without tool calls are passed through unchanged
    #[tokio::test]
    async fn test_with_tool_calls_no_tool_calls() {
        let messages = vec![
            ChatCompletionMessage::default().content("test message"),
            ChatCompletionMessage::default().content("another message"),
        ];

        let actual = stream::iter(messages.clone().into_iter().map(Ok))
            .boxed()
            .with_tool_calls()
            .map(Result::unwrap)
            .collect::<Vec<_>>()
            .await;

        let expected = messages;
        assert_eq!(actual, expected);
    }

    /// Tests error handling when receiving invalid JSON in tool call arguments
    #[tokio::test]
    async fn test_with_tool_calls_invalid_parts() {
        let messages = vec![ChatCompletionMessage::default()
            .add_tool_call(
                ToolCallPart::default()
                    .name(ToolName::new("test_tool"))
                    .arguments_part("{invalid json"),
            )
            .finish_reason_opt(Some(FinishReason::ToolCalls))];

        let actual = stream::iter(messages.into_iter().map(Ok))
            .boxed()
            .with_tool_calls()
            .collect::<Vec<_>>()
            .await;

        assert_eq!(actual.len(), 2);
        assert!(actual[1].is_err());
    }

    /// Tests error handling when a tool call is missing the required name field
    #[tokio::test]
    async fn test_with_tool_calls_missing_name() {
        let messages = vec![ChatCompletionMessage::default()
            .add_tool_call(ToolCallPart::default().arguments_part("{\"key\": \"value\"}"))
            .finish_reason_opt(Some(FinishReason::ToolCalls))];

        let actual = stream::iter(messages.into_iter().map(Ok))
            .boxed()
            .with_tool_calls()
            .collect::<Vec<_>>()
            .await;

        assert_eq!(actual.len(), 2);
        assert!(matches!(actual[1], Err(Error::ToolUseMissingName)));
    }
}
