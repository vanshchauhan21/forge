use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Response {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: u64,
    pub model: String,
    pub object: String,
    pub system_fingerprint: Option<String>,
    pub usage: Option<ResponseUsage>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Choice {
    NonChatChoice(NonChatChoice),
    NonStreamingChoice(NonStreamingChoice),
    StreamingChoice(StreamingChoice),
}

#[derive(Deserialize, Debug)]
pub struct NonChatChoice {
    pub finish_reason: Option<String>,
    pub text: String,
    pub error: Option<ErrorResponse>,
}

#[derive(Deserialize, Debug)]
pub struct NonStreamingChoice {
    pub finish_reason: Option<String>,
    pub message: Message,
    pub error: Option<ErrorResponse>,
}

#[derive(Deserialize, Debug)]
pub struct StreamingChoice {
    pub finish_reason: Option<String>,
    pub delta: Delta,
    pub error: Option<ErrorResponse>,
}

#[derive(Deserialize, Debug)]
pub struct Message {
    pub content: Option<String>,
    pub role: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Debug)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Debug)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Deserialize, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
