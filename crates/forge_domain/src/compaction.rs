// FIXME: compaction should be expose as a service via forge_app instead of
// being core to the domain.
use std::cmp::min;
use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use tracing::{debug, info};

use crate::{
    extract_tag_content, Agent, ChatCompletionMessage, Compact, Context, ContextMessage,
    ProviderService, Role, Services, TemplateService,
};

/// Handles the compaction of conversation contexts to manage token usage
#[derive(Clone)]
pub struct ContextCompactor<Services> {
    services: Arc<Services>,
}

impl<S: Services> ContextCompactor<S> {
    /// Creates a new ContextCompactor instance
    pub fn new(services: Arc<S>) -> Self {
        Self { services }
    }

    /// Check if compaction is needed and compact the context if so
    pub async fn compact_context(&self, agent: &Agent, context: Context) -> Result<Context> {
        // Early return if compaction not needed

        if let Some(ref compact) = agent.compact {
            // Ensure that compaction conditions are met
            if !compact.should_compact(&context) {
                return Ok(context);
            }

            debug!(agent_id = %agent.id, "Context compaction triggered");

            // Identify and compress the first compressible sequence
            // Get all compressible sequences, considering the preservation window
            match find_sequence(&context, compact.retention_window)
                .into_iter()
                .next()
            {
                Some(sequence) => {
                    self.compress_single_sequence(compact, context, sequence)
                        .await
                }
                None => {
                    debug!(agent_id = %agent.id, "No compressible sequences found");
                    Ok(context)
                }
            }
        } else {
            Ok(context)
        }
    }

    /// Compress a single identified sequence of assistant messages
    async fn compress_single_sequence(
        &self,
        compact: &Compact,
        mut context: Context,
        sequence: (usize, usize),
    ) -> Result<Context> {
        let (start, end) = sequence;

        // Extract the sequence to summarize
        let sequence_messages = &context.messages[start..=end];

        // Generate summary for this sequence
        let summary = self
            .generate_summary_for_sequence(compact, sequence_messages)
            .await?;

        // Log the summary for debugging
        info!(
            summary = %summary,
            sequence_start = sequence.0,
            sequence_end = sequence.1,
            "Created context compaction summary"
        );

        // Replace the sequence with a single summary message using splice
        // This removes the sequence and inserts the summary message in-place
        context.messages.splice(
            start..=end,
            std::iter::once(ContextMessage::assistant(summary, None)),
        );

        Ok(context)
    }

    /// Generate a summary for a specific sequence of assistant messages
    async fn generate_summary_for_sequence(
        &self,
        compact: &Compact,
        messages: &[ContextMessage],
    ) -> Result<String> {
        // Create a temporary context with just the sequence for summarization
        let sequence_context = messages
            .iter()
            .fold(Context::default(), |ctx, msg| ctx.add_message(msg.clone()));

        // Render the summarization prompt
        let prompt = self
            .services
            .template_service()
            .render_summarization(compact, &sequence_context)
            .await?;

        // Create a new context
        let mut context = Context::default().add_message(ContextMessage::user(prompt));

        // Set max_tokens for summary
        if let Some(max_token) = compact.max_tokens {
            context = context.max_tokens(max_token);
        }

        // Get summary from the provider
        let response = self
            .services
            .provider_service()
            .chat(&compact.model, context)
            .await?;

        self.collect_completion_stream_content(compact, response)
            .await
    }

    /// Collects the content from a streaming ChatCompletionMessage response
    /// and extracts text within the configured tag if present
    async fn collect_completion_stream_content<T>(
        &self,
        compact: &Compact,
        mut stream: T,
    ) -> Result<String>
    where
        T: futures::Stream<Item = Result<ChatCompletionMessage>> + Unpin,
    {
        let mut result_content = String::new();

        while let Some(message_result) = stream.next().await {
            let message = message_result?;
            if let Some(content) = message.content {
                result_content.push_str(content.as_str());
            }
        }

        // Extract content from within configured tags if present and if tag is
        // configured
        if let Some(extracted) = extract_tag_content(
            &result_content,
            compact
                .summary_tag
                .as_ref()
                .cloned()
                .unwrap_or_default()
                .as_str(),
        ) {
            return Ok(extracted.to_string());
        }

        // If no tag extraction performed, return the original content
        Ok(result_content)
    }
}

/// Finds all valid compressible sequences in the context, respecting the
/// preservation window
fn find_sequence(context: &Context, preserve_last_n: usize) -> Option<(usize, usize)> {
    let messages = &context.messages;
    if messages.is_empty() {
        return None;
    }

    // len will be always > 0
    let length = messages.len();
    let mut max_len = length - min(length, preserve_last_n);

    if max_len == 0 {
        return None;
    }

    // Additional check: if max_len < 1, we can't safely do max_len - 1
    if max_len < 1 {
        return None;
    }
    if messages
        .get(max_len - 1)
        .is_some_and(|msg| msg.has_tool_call())
    {
        max_len -= 1;
    }

    let user_messages = messages
        .iter()
        .enumerate()
        .take(max_len)
        .filter(|(_, message)| message.has_role(Role::User))
        .collect::<Vec<_>>();

    // If there are no user messages, there can't be any sequences
    if user_messages.is_empty() {
        return None;
    }
    let start_positions = user_messages
        .iter()
        .map(|(start, _)| min(start.saturating_add(1), max_len.saturating_sub(1)))
        .collect::<Vec<_>>();

    let mut end_positions = user_messages
        .iter()
        .skip(1)
        .map(|(pos, _)| pos.saturating_sub(1))
        .collect::<Vec<_>>();
    end_positions.push(max_len - 1);

    // If either vector is empty, there can't be any compressible sequences
    if start_positions.is_empty() || end_positions.is_empty() {
        return None;
    }

    start_positions
        .iter()
        .zip(end_positions.iter())
        .find(|(start, end)| *end > *start)
        .map(|(a, b)| (*a, *b))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::{ToolCallFull, ToolCallId, ToolName, ToolResult};

    #[test]
    fn test_identify_first_compressible_sequence() {
        // Create a context with a sequence of assistant messages
        let context = Context::default()
            .add_message(ContextMessage::system("System message"))
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::assistant("Assistant message 1", None))
            .add_message(ContextMessage::assistant("Assistant message 2", None))
            .add_message(ContextMessage::assistant("Assistant message 3", None))
            .add_message(ContextMessage::user("User message 2"))
            .add_message(ContextMessage::assistant("Assistant message 4", None));

        // The first sequence is from index 2 to 4 (assistant messages 1, 2, and 3)
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());

        let (start, end) = sequence.unwrap();
        assert_eq!(start, 2);
        assert_eq!(end, 4);
    }

    #[test]
    fn test_no_compressible_sequence() {
        // Create a context with no sequence of multiple assistant messages
        let context = Context::default()
            .add_message(ContextMessage::system("System message"))
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::assistant("Assistant message 1", None))
            .add_message(ContextMessage::user("User message 2"))
            .add_message(ContextMessage::assistant("Assistant message 2", None))
            .add_message(ContextMessage::user("User message 3"))
            .add_message(ContextMessage::assistant("Assistant message 3", None));

        // There are no sequences of multiple assistant messages
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_none());
    }

    #[test]
    fn test_sequence_at_end_of_context() {
        // Create a context with a sequence at the end
        let context = Context::default()
            .add_message(ContextMessage::system("System message")) // 0
            .add_message(ContextMessage::user("User message 1")) // 1
            .add_message(ContextMessage::assistant("Assistant message 1", None)) // 2
            .add_message(ContextMessage::user("User message 2")) // 3
            .add_message(ContextMessage::assistant("Assistant message 2", None)) // 4
            .add_message(ContextMessage::assistant("Assistant message 3", None)); // 5

        // The sequence is at the end (indices 4-5)
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());

        let (start, end) = sequence.unwrap();
        assert_eq!(start, 4);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_identify_sequence_with_tool_calls() {
        // Create a context with assistant messages containing tool calls
        let tool_call = ToolCallFull {
            name: ToolName::new("tool_forge_fs_read"),
            call_id: Some(ToolCallId::new("call_123")),
            arguments: json!({"path": "/test/path"}),
        };

        let context = Context::default()
            .add_message(ContextMessage::system("System message"))
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::assistant(
                "Assistant message with tool call",
                Some(vec![tool_call.clone()]),
            ))
            .add_message(ContextMessage::assistant(
                "Assistant message with another tool call",
                Some(vec![tool_call.clone()]),
            ))
            .add_message(ContextMessage::user("User message 2"));

        // The sequence is from index 2 to 3 (both assistant messages with tool calls)
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());

        let (start, end) = sequence.unwrap();
        assert_eq!(start, 2);
        assert_eq!(end, 3);
    }

    #[test]
    fn test_identify_sequence_with_tool_results() {
        // Create a context with assistant messages and tool results
        let tool_call = ToolCallFull {
            name: ToolName::new("tool_forge_fs_read"),
            call_id: Some(ToolCallId::new("call_123")),
            arguments: json!({"path": "/test/path"}),
        };

        let tool_result = ToolResult::new(ToolName::new("tool_forge_fs_read"))
            .call_id(ToolCallId::new("call_123"))
            .success(json!({"content": "File content"}).to_string());

        let context = Context::default()
            .add_message(ContextMessage::system("System message"))
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::assistant(
                "Assistant message with tool call",
                Some(vec![tool_call]),
            ))
            .add_message(ContextMessage::tool_result(tool_result))
            .add_message(ContextMessage::assistant(
                "Assistant follow-up message",
                None,
            ))
            .add_message(ContextMessage::assistant("Another assistant message", None))
            .add_message(ContextMessage::user("User message 2"));

        // Now tool results are considered compressible
        // The sequence is from index 2 to 5 (assistant + tool + 2 assistant messages)
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());

        let (start, end) = sequence.unwrap();
        assert_eq!(start, 2);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_mixed_assistant_and_tool_messages() {
        // Create a context where we strictly alternate assistant/tool and user messages
        let tool_call1 = ToolCallFull {
            name: ToolName::new("tool_forge_fs_read"),
            call_id: Some(ToolCallId::new("call_123")),
            arguments: json!({"path": "/test/path1"}),
        };

        let tool_call2 = ToolCallFull {
            name: ToolName::new("tool_forge_fs_search"),
            call_id: Some(ToolCallId::new("call_456")),
            arguments: json!({"path": "/test/path2", "regex": "pattern"}),
        };

        let tool_result1 = ToolResult::new(ToolName::new("tool_forge_fs_read"))
            .call_id(ToolCallId::new("call_123"))
            .success(json!({"content": "File content 1"}).to_string());

        let tool_result2 = ToolResult::new(ToolName::new("tool_forge_fs_search"))
            .call_id(ToolCallId::new("call_456"))
            .success(json!({"matches": ["match1", "match2"]}).to_string());

        // Create a context where we strictly alternate assistant and non-assistant
        // messages to ensure no compressible sequence forms
        let context = Context::default()
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::assistant(
                "Assistant message with tool call",
                Some(vec![tool_call1]),
            ))
            .add_message(ContextMessage::tool_result(tool_result1))
            .add_message(ContextMessage::user("User follow-up question"))
            .add_message(ContextMessage::assistant(
                "Assistant with another tool call",
                Some(vec![tool_call2]),
            ))
            .add_message(ContextMessage::tool_result(tool_result2))
            .add_message(ContextMessage::user("User message 2"));

        // With the new logic, we now have a compressible sequence from index 1-2
        // (assistant + tool result)
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());

        let (start, end) = sequence.unwrap();
        assert_eq!(start, 1);
        assert_eq!(end, 2);
    }

    #[test]
    fn test_consecutive_assistant_messages_with_tools() {
        // Test when we have consecutive assistant messages with tool calls
        // followed by tool results but the assistant messages themselves are
        // consecutive
        let tool_call1 = ToolCallFull {
            name: ToolName::new("tool_forge_fs_read"),
            call_id: Some(ToolCallId::new("call_123")),
            arguments: json!({"path": "/test/path1"}),
        };

        let tool_call2 = ToolCallFull {
            name: ToolName::new("tool_forge_fs_search"),
            call_id: Some(ToolCallId::new("call_456")),
            arguments: json!({"path": "/test/path2", "regex": "pattern"}),
        };

        let tool_result1 = ToolResult::new(ToolName::new("tool_forge_fs_read"))
            .call_id(ToolCallId::new("call_123"))
            .success(json!({"content": "File content 1"}).to_string());

        let tool_result2 = ToolResult::new(ToolName::new("tool_forge_fs_search"))
            .call_id(ToolCallId::new("call_456"))
            .success(json!({"matches": ["match1", "match2"]}).to_string());

        let context = Context::default()
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::assistant(
                "Assistant message with tool call",
                Some(vec![tool_call1.clone()]),
            ))
            .add_message(ContextMessage::assistant(
                "Another assistant message",
                Some(vec![tool_call2.clone()]),
            ))
            .add_message(ContextMessage::assistant("Third assistant message", None))
            .add_message(ContextMessage::tool_result(tool_result1))
            .add_message(ContextMessage::tool_result(tool_result2))
            .add_message(ContextMessage::user("User message 2"));

        // The sequence now includes both assistant messages and tool results (indices
        // 1-5)
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());

        let (start, end) = sequence.unwrap();
        assert_eq!(start, 1);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_only_tool_results() {
        // Test when we have just tool results in sequence
        let tool_result1 = ToolResult::new(ToolName::new("tool_forge_fs_read"))
            .call_id(ToolCallId::new("call_123"))
            .success(json!({"content": "File content 1"}).to_string());

        let tool_result2 = ToolResult::new(ToolName::new("tool_forge_fs_search"))
            .call_id(ToolCallId::new("call_456"))
            .success(json!({"matches": ["match1", "match2"]}).to_string());

        let context = Context::default()
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::tool_result(tool_result1))
            .add_message(ContextMessage::tool_result(tool_result2))
            .add_message(ContextMessage::user("User message 2"));

        // The sequence is the two tool results (indices 1-2)
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());

        let (start, end) = sequence.unwrap();
        assert_eq!(start, 1);
        assert_eq!(end, 2);
    }

    #[test]
    fn test_mixed_assistant_and_single_tool() {
        // Create a context with an assistant message and a tool result,
        // but each preceded by user messages so they're not consecutive
        let tool_call = ToolCallFull {
            name: ToolName::new("tool_forge_fs_read"),
            call_id: Some(ToolCallId::new("call_123")),
            arguments: json!({"path": "/test/path"}),
        };

        let tool_result = ToolResult::new(ToolName::new("tool_forge_fs_read"))
            .call_id(ToolCallId::new("call_123"))
            .success(json!({"content": "File content 1"}).to_string());

        let context = Context::default()
            .add_message(ContextMessage::user("User message 1")) // 0
            .add_message(ContextMessage::assistant(
                "Assistant message with tool call",
                Some(vec![tool_call]),
            )) // 1
            .add_message(ContextMessage::user("User intermediate message")) // 2
            .add_message(ContextMessage::tool_result(tool_result)) // 3
            .add_message(ContextMessage::user("User message 2")); // 4

        // No compressible sequence as each potential message is separated by a user
        // message
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_none());
    }
    #[test]
    fn test_preserve_last_n_messages() {
        // Create a context with multiple sequences that could be compressed
        let context = Context::default()
            .add_message(ContextMessage::system("System message"))
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::assistant("Assistant message 1", None)) // 2
            .add_message(ContextMessage::assistant("Assistant message 2", None)) // 3
            .add_message(ContextMessage::assistant("Assistant message 3", None)) // 4
            .add_message(ContextMessage::user("User message 2")) // 5
            .add_message(ContextMessage::assistant("Assistant message 4", None)) // 6
            .add_message(ContextMessage::assistant("Assistant message 5", None)); // 7

        // Without preservation, we'd compress messages 2-4
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());
        let (start, end) = sequence.unwrap();
        assert_eq!(start, 2);
        assert_eq!(end, 4);

        // With preserve_last_n = 3, we should preserve the last 3 messages (indices 5,
        // 6, 7) So we should still get the sequence at 2-4
        let sequence = find_sequence(&context, 3);
        assert!(sequence.is_some());
        let (start, end) = sequence.unwrap();
        assert_eq!(start, 2);
        assert_eq!(end, 4);

        // With preserve_last_n = 5, we should preserve indices 3-7
        // So we should get no compressible sequence, since we can only consider indices
        // 0-2
        let sequence = find_sequence(&context, 5);
        assert!(sequence.is_none());

        // With preserve_last_n = 8 (more than total messages), we should get no
        // compressible sequence
        let sequence = find_sequence(&context, 8);
        assert!(sequence.is_none());
    }
    #[test]
    fn test_preserve_last_n_with_sequence_at_end() {
        // Create a context with a sequence at the end
        let context = Context::default()
            .add_message(ContextMessage::system("System message")) // 0
            .add_message(ContextMessage::user("User message 1")) // 1
            .add_message(ContextMessage::assistant("Assistant message 1", None)) // 2
            .add_message(ContextMessage::user("User message 2")) // 3
            .add_message(ContextMessage::assistant("Assistant message 2", None)) // 4
            .add_message(ContextMessage::assistant("Assistant message 3", None)) // 5
            .add_message(ContextMessage::assistant("Assistant message 4", None)); // 6

        // Without preservation, we'd compress the sequence at indices 4-6
        let sequence = find_sequence(&context, 0);
        assert!(sequence.is_some());
        let (start, end) = sequence.unwrap();
        assert_eq!(start, 4);
        assert_eq!(end, 6);

        // With preserve_last_n = 2, we should preserve indices 5-6
        // So the compressible sequence should be index 4 only, which is not enough for
        // compression
        let sequence = find_sequence(&context, 2);
        assert!(sequence.is_none());

        // With preserve_last_n = 1, we should preserve index 6
        // So the compressible sequence should be indices 4-5
        let sequence = find_sequence(&context, 1);
        assert!(sequence.is_some());
        let (start, end) = sequence.unwrap();
        assert_eq!(start, 4);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_preserve_tool_call_atomicity() {
        let tool_calls = Some(vec![ToolCallFull {
            name: ToolName::new("tool_forge_fs_read"),
            call_id: None,
            arguments: json!({"path": "/test/path"}),
        }]);

        let tool_results = vec![ToolResult::new(ToolName::new("tool_forge_fs_read"))
            .call_id(ToolCallId::new("call_123"))
            .success(json!({"content": "File content 1"}).to_string())];

        // Create a context with a sequence at the end
        let context = Context::default()
            .add_message(ContextMessage::system("System message")) // 0
            .add_message(ContextMessage::user("User Message 1")) // 1
            .add_message(ContextMessage::assistant(
                "Assistant Message 1",
                tool_calls.clone(),
            )) // 2
            .add_tool_results(tool_results.clone()) // 3
            .add_message(ContextMessage::assistant(
                "Assistant Message 2",
                tool_calls.clone(),
            )) // 4
            .add_tool_results(tool_results.clone()) // 5
            .add_message(ContextMessage::assistant(
                "Assistant Message 3",
                tool_calls.clone(),
            )) // 6
            .add_tool_results(tool_results.clone()) // 7
            .add_message(ContextMessage::assistant(
                "Assistant Message 4",
                tool_calls.clone(),
            )) // 8
            .add_tool_results(tool_results.clone()); // 9

        // All the messages should be considered
        let sequence = find_sequence(&context, 0).unwrap();
        assert_eq!(sequence, (2, 9));

        // Since we can not break in between a tool call is corresponding tool-result
        let sequence = find_sequence(&context, 1).unwrap();
        assert_eq!(sequence, (2, 7));

        let sequence = find_sequence(&context, 2).unwrap();
        assert_eq!(sequence, (2, 7));
    }

    #[test]
    fn test_empty_context() {
        // Test edge case: an empty context
        let context = Context::default();
        let result = find_sequence(&context, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_single_message_context() {
        // Test edge case: context with only one message
        let context = Context::default().add_message(ContextMessage::system("System message"));
        let result = find_sequence(&context, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_preserve_equals_length() {
        // Test edge case: preservation window equals message count
        let context = Context::default()
            .add_message(ContextMessage::system("System message"))
            .add_message(ContextMessage::user("User message"))
            .add_message(ContextMessage::assistant("Assistant message", None));

        // Context has 3 messages, preserve_last_n = 3
        let result = find_sequence(&context, 3);
        assert!(result.is_none());
    }

    #[test]
    fn test_max_len_zero_after_tool_call() {
        // Test edge case: max_len becomes 0 after tool call adjustment
        // Create a context with 2 messages where the second one has a tool call
        let tool_call = ToolCallFull {
            name: ToolName::new("tool_forge_fs_read"),
            call_id: Some(ToolCallId::new("call_123")),
            arguments: json!({"path": "/test/path"}),
        };

        let context = Context::default()
            .add_message(ContextMessage::user("User message"))
            .add_message(ContextMessage::assistant(
                "Assistant message with tool call",
                Some(vec![tool_call]),
            ));

        // With preserve_last_n = 0, max_len = 2, but after tool call adjustment it
        // could become 1 which might lead to underflow in some parts of the
        // code
        let result = find_sequence(&context, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_start_end_positions() {
        // Test edge case: empty start/end positions
        // Create a context with only system and user messages (no assistant messages)
        // which would result in empty start/end position vectors
        let context = Context::default()
            .add_message(ContextMessage::system("System message"))
            .add_message(ContextMessage::user("User message 1"))
            .add_message(ContextMessage::user("User message 2"))
            .add_message(ContextMessage::user("User message 3"));

        let result = find_sequence(&context, 0);
        assert!(result.is_none());
    }

    #[test]
    fn test_potential_underflow_edge_cases() {
        // Test edge case: potential integer underflow scenarios

        // Case 1: preserve_last_n = 1, total messages = 2, with the last message having
        // a tool call
        let tool_call = ToolCallFull {
            name: ToolName::new("tool_forge_fs_read"),
            call_id: Some(ToolCallId::new("call_123")),
            arguments: json!({"path": "/test/path"}),
        };

        let context = Context::default()
            .add_message(ContextMessage::user("User message"))
            .add_message(ContextMessage::assistant(
                "Assistant message with tool call",
                Some(vec![tool_call]),
            ));

        // With preserve_last_n = 1, max_len = 2-1 = 1,
        // then if we try to check messages[max_len-1] this could cause underflow if not
        // handled
        let result = find_sequence(&context, 1);
        assert!(result.is_none());

        // Case 2: Context with exactly 2 messages (user, assistant)
        let context = Context::default()
            .add_message(ContextMessage::user("User message"))
            .add_message(ContextMessage::assistant("Assistant message", None));

        // With preserve_last_n = 0, max_len = 2, but we need at least 3 messages for
        // compression
        let result = find_sequence(&context, 0);
        assert!(result.is_none());
    }
}
