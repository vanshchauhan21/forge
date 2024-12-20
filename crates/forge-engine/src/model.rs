use forge_provider::model::{AnyMessage, Assistant, Message, Request, User};
use forge_tool::Tool;
use serde_json::Value;

use crate::File;

#[derive(Default)]
pub struct State {
    stack: Vec<Request>,
    context: Request,
    history: Vec<AnyMessage>,
}

#[derive(Default)]
pub enum Command {
    LLMRequest(Request),
    UseTool(ToolRequest),

    #[default]
    Empty,
}

#[derive(Clone, Default)]
pub struct ToolRequest {
    name: String,
    value: Value,
}

#[derive(Clone)]
pub struct ToolResponse {
    name: String,
    value: std::result::Result<String, String>,
}

pub enum Action {
    Prompt(Message<User>),
    Initialize,
    LLMResponse {
        message: Message<Assistant>,
        use_tool: Option<ToolRequest>,
    },
    ToolResponse(ToolResponse),
}

impl From<ToolResponse> for Message<User> {
    fn from(tool_response: ToolResponse) -> Self {
        todo!()
    }
}

pub enum Event {
    Ask(Option<String>),
    Say(String),
    Err(String),
}

fn insert_into<T>(vector: Option<Vec<T>>, value: T) -> Option<Vec<T>> {
    match vector {
        Some(mut vec) => {
            vec.push(value);
            Some(vec)
        }
        None => Some(vec![value]),
    }
}

impl From<File> for forge_provider::model::Message<User> {
    fn from(value: File) -> Self {
        Message::user(format!("{}\n{}", value.path, value.content))
    }
}
