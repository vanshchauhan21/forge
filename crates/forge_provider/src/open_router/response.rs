use std::collections::HashMap;
use std::str::FromStr;

use forge_domain::{
    ChatCompletionMessage as ModelResponse, Content, FinishReason, ToolCallFull, ToolCallId,
    ToolCallPart, ToolName,
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenRouterResponse {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub created: u64,
    pub object: String,
    pub system_fingerprint: Option<String>,
    pub usage: Option<ResponseUsage>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Choice {
    NonChat {
        finish_reason: Option<String>,
        text: String,
        error: Option<ErrorResponse>,
    },
    NonStreaming {
        logprobs: Option<serde_json::Value>,
        index: u32,
        finish_reason: Option<String>,
        message: ResponseMessage,
        error: Option<ErrorResponse>,
    },
    Streaming {
        finish_reason: Option<String>,
        delta: ResponseMessage,
        error: Option<ErrorResponse>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ErrorResponse {
    pub code: u32,
    pub message: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseMessage {
    pub content: Option<String>,
    pub role: Option<String>,
    pub tool_calls: Option<Vec<OpenRouterToolCall>>,
    pub refusal: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenRouterToolCall {
    pub id: Option<ToolCallId>,
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionCall {
    // Only the first event typically has the name of the function call
    pub name: Option<ToolName>,
    pub arguments: String,
}

impl TryFrom<OpenRouterResponse> for ModelResponse {
    type Error = Error;

    fn try_from(res: OpenRouterResponse) -> Result<Self, Self::Error> {
        if let Some(choice) = res.choices.first() {
            let response = match choice {
                Choice::NonChat { text, finish_reason, .. } => {
                    ModelResponse::assistant(Content::full(text)).finish_reason_opt(
                        finish_reason
                            .clone()
                            .and_then(|s| FinishReason::from_str(&s).ok()),
                    )
                }
                Choice::NonStreaming { message, finish_reason, .. } => {
                    let mut resp = ModelResponse::assistant(Content::full(
                        message.content.clone().unwrap_or_default(),
                    ))
                    .finish_reason_opt(
                        finish_reason
                            .clone()
                            .and_then(|s| FinishReason::from_str(&s).ok()),
                    );
                    if let Some(tool_calls) = &message.tool_calls {
                        for tool_call in tool_calls {
                            resp = resp.add_tool_call(ToolCallFull {
                                call_id: tool_call.id.clone(),
                                name: tool_call
                                    .function
                                    .name
                                    .clone()
                                    .ok_or(Error::ToolCallMissingName)?,
                                arguments: serde_json::from_str(&tool_call.function.arguments)?,
                            });
                        }
                    }
                    resp
                }
                Choice::Streaming { delta, finish_reason, .. } => {
                    let mut resp = ModelResponse::assistant(Content::part(
                        delta.content.clone().unwrap_or_default(),
                    ))
                    .finish_reason_opt(
                        finish_reason
                            .clone()
                            .and_then(|s| FinishReason::from_str(&s).ok()),
                    );
                    if let Some(tool_calls) = &delta.tool_calls {
                        for tool_call in tool_calls {
                            resp = resp.add_tool_call(ToolCallPart {
                                call_id: tool_call.id.clone(),
                                name: tool_call.function.name.clone(),
                                arguments_part: tool_call.function.arguments.clone(),
                            });
                        }
                    }
                    resp
                }
            };

            Ok(response)
        } else {
            Err(Error::EmptyContent)
        }
    }
}
