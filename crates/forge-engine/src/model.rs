use std::rc::Rc;

use derive_more::derive::From;
use derive_setters::Setters;
use forge_provider::model::{ContentPart, Request, TextContent};
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

pub trait Role {
    fn name() -> String;
}

impl Role for System {
    fn name() -> String {
        "system".to_string()
    }
}

impl Role for User {
    fn name() -> String {
        "user".to_string()
    }
}

impl Role for Assistant {
    fn name() -> String {
        "assistant".to_string()
    }
}

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

fn insert_into<T>(vector: Option<Vec<T>>, value: T) -> Option<Vec<T>> {
    match vector {
        Some(mut vec) => {
            vec.push(value);
            Some(vec)
        }
        None => Some(vec![value]),
    }
}

impl From<Context> for Request {
    fn from(value: Context) -> Self {
        let mut request = Request::default();
        // Add System Message [DONE]
        request.messages = insert_into(request.messages, value.system.into());

        // Add Add all tools
        request.tools = Some(
            value
                .tools
                .iter()
                .flat_map(|tool| into_tool(tool.as_ref()))
                .collect(),
        );

        // Encourage the model to use tools
        request.tool_choice = Some(forge_provider::model::ToolChoice::Auto);

        // Add User Message
        // Add Context Files
        request
    }
}

impl<R: Role> From<Message<R>> for forge_provider::model::Message {
    fn from(value: Message<R>) -> Self {
        forge_provider::model::Message {
            role: R::name(),
            content: ContentPart::Text(TextContent {
                r#type: "text".to_string(),
                text: value.content,
            }),
            name: None,
        }
    }
}

fn into_tool(tool: &dyn Tool) -> Vec<forge_provider::model::Tool> {
    tool.tools_list()
        .tools
        .iter()
        .map(|tool| forge_provider::model::Tool {
            r#type: "function".to_string(),
            function: forge_provider::model::FunctionDescription {
                description: tool.description.clone(),
                name: tool.name.clone(),
                parameters: tool.input_schema.clone(),
            },
        })
        .collect()
}
