use derive_more::derive::{Display, From};
use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use super::{ToolCall, ToolResult};

/// Represents a message being sent to the LLM provider
/// NOTE: ToolResults message are part of the larger Request object and not part
/// of the message.
#[derive(Clone, Debug, Deserialize, From, PartialEq, Serialize)]
pub enum CompletionMessage {
    ContentMessage(ContentMessage),
    ToolMessage(ToolResult),
}

impl CompletionMessage {
    pub fn user(content: impl ToString) -> Self {
        ContentMessage {
            role: Role::User,
            content: content.to_string(),
            tool_call: None,
        }
        .into()
    }

    pub fn system(content: impl ToString) -> Self {
        ContentMessage {
            role: Role::System,
            content: content.to_string(),
            tool_call: None,
        }
        .into()
    }

    pub fn assistant(content: impl ToString) -> Self {
        ContentMessage {
            role: Role::Assistant,
            content: content.to_string(),
            tool_call: None,
        }
        .into()
    }

    pub fn content(&self) -> String {
        match self {
            CompletionMessage::ContentMessage(message) => message.content.to_string(),
            CompletionMessage::ToolMessage(result) => {
                serde_json::to_string(&result.content).unwrap()
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Setters)]
#[setters(strip_option, into)]
pub struct ContentMessage {
    pub role: Role,
    pub content: String,

    // FIXME: Message could contain multiple tool calls
    pub tool_call: Option<ToolCall>,
}

impl ContentMessage {
    pub fn assistant(content: impl ToString) -> Self {
        Self {
            role: Role::Assistant,
            content: content.to_string(),
            tool_call: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Display)]
pub enum Role {
    System,
    User,
    Assistant,
}
