use std::rc::Rc;

use derive_more::derive::From;
use derive_setters::Setters;
use forge_provider::model::Request;
use forge_tool::Tool;
use serde_json::Value;

use crate::File;

#[derive(Default)]
pub struct State {
    stack: Vec<Context>,
    context: Context,
    history: Vec<AnyMessage>,
}

#[derive(Clone, Default)]
pub struct System;
#[derive(Clone, Default)]
pub struct User;
#[derive(Clone, Default)]
pub struct Assistant;

#[derive(Clone, From)]
pub enum AnyMessage {
    User(Message<User>),
    Assistant(Message<Assistant>),
}

#[derive(Clone, Default)]
pub struct Message<Role> {
    content: String,
    role: Role,
}

impl Message<System> {
    pub fn system(content: String) -> Self {
        Message {
            content,
            role: System,
        }
    }
}

impl Message<User> {
    pub fn user(content: String) -> Self {
        Message {
            content,
            role: User,
        }
    }
}

impl Message<Assistant> {
    pub fn assistant(content: String) -> Self {
        Message {
            content,
            role: Assistant,
        }
    }
}

#[derive(Default, Clone, Setters)]
pub struct Context {
    pub system: Message<System>,
    pub message: Vec<AnyMessage>,
    pub tools: Vec<Rc<dyn Tool>>,
    pub files: Vec<File>,
}

impl Context {
    pub fn add_message(mut self, message: impl Into<AnyMessage>) -> Self {
        self.message.push(message.into());
        self
    }

    pub fn new(system: Message<System>) -> Self {
        Context {
            system,
            message: Vec::new(),
            tools: Vec::new(),
            files: Vec::new(),
        }
    }
}

#[derive(Default)]
pub enum Command {
    LLMRequest(Context),
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
    Inquire(Option<String>),
    Text(String),
    Error(String),
    End,
}

impl From<Context> for Request {
    fn from(value: Context) -> Self {
        todo!()
    }
}
