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

// TODO: use provider::model here
#[derive(Default, Clone, Setters)]
pub struct Context {
    pub system: Message<System>,
    pub messages: Vec<AnyMessage>,
    pub tools: Vec<Rc<dyn Tool>>,
    pub files: Vec<File>,
}

impl Context {
    pub fn add_message(mut self, message: impl Into<AnyMessage>) -> Self {
        self.messages.push(message.into());
        self
    }

    pub fn new(system: Message<System>) -> Self {
        Context {
            system,
            messages: Vec::new(),
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

impl From<Context> for Request {
    fn from(value: Context) -> Self {
        let mut request = Request::default().add_message(value.system);
        // Add System Message
        request.context = insert_into(request.context, value.system.into());

        // Add User Message
        for message in value.messages {
            request.context = insert_into(request.context, message.into());
        }

        // Add Context Files
        for file in value.files {
            request.context = insert_into(request.context, file.into());
        }

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

        request
    }
}

impl<R: Role> From<Message<R>> for forge_provider::model::Message {
    fn from(value: Message<R>) -> Self {
        forge_provider::model::Message::default()
        .content(value.content)
        .role(R::name())
    }
}

impl From<AnyMessage> for forge_provider::model::Message {
    fn from(value: AnyMessage) -> Self {
        match value {
            AnyMessage::User(message) => message.into(),
            AnyMessage::Assistant(message) => message.into(),
        }
    }
}

fn into_tool(tool: &dyn Tool) -> Vec<forge_provider::model::Tool> {
    tool.tools_list()
        .tools
        .iter()
        .map(|tool| {
            
            forge_provider::model::Tool {
                id: forge_provider::model::ToolId(tool.id().to_string()),
                description: tool.description(),
                input_schema: forge_provider::model::JsonSchema::from(tool.input_schema()),
                output_schema: None,
    
        }
        })
        .collect()
}

impl From<File> for forge_provider::model::Message {
    fn from(value: File) -> Self {
        forge_provider::model::Message::default()
        .content(format!("{}\n{}", value.path, value.content))
        .role(User::name())
    }
}
