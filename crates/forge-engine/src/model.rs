use derive_more::derive::From;
use serde_json::Value;

pub struct App {
    stack: Vec<Context>,
    context: Context,
    history: Vec<AnyMessage>,
}

#[derive(Clone)]
pub struct System;
#[derive(Clone)]
pub struct User;
#[derive(Clone)]
pub struct Assistant;

#[derive(Clone, From)]
pub enum AnyMessage {
    User(Message<User>),
    Assistant(Message<Assistant>),
}

#[derive(Clone)]
pub struct Message<Role> {
    content: String,
    role: Role,
}

impl Message<System> {
    pub fn new(content: &str) -> Self {
        Message {
            content: content.to_string(),
            role: System,
        }
    }
}

#[derive(Clone)]
pub struct Context {
    system: Message<System>,
    message: Vec<AnyMessage>,
}

impl Context {
    pub fn push(&mut self, message: impl Into<AnyMessage>) {
        self.message.push(message.into());
    }

    pub fn new(system: Message<System>) -> Self {
        Context {
            system,
            message: Vec::new(),
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

#[derive(Clone)]
pub struct ToolRequest {
    name: String,
    value: Value,
}

#[derive(Clone)]
pub struct ToolResponse {
    name: String,
    value: Result<Value, Value>,
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

impl App {
    fn command(&mut self, action: Action) -> Command {
        match action {
            Action::Initialize => {
                self.context.system = Message::new("<SYSTEM MESSAGE>");
                Command::default()
            }
            Action::Prompt(message) => {
                self.history.push(message.clone().into());
                self.context.push(message.clone());
                Command::LLMRequest(self.context.clone())
            }
            Action::LLMResponse { message, use_tool } => {
                self.history.push(message.clone().into());
                self.context.push(message.clone());
                if let Some(tool) = use_tool {
                    Command::UseTool(tool)
                } else {
                    Command::default()
                }
            }
            Action::ToolResponse(tool_response) => match &tool_response.value {
                Ok(_) => {
                    self.context.push(Message::<User>::from(tool_response));
                    Command::LLMRequest(self.context.clone())
                }
                Err(value) => todo!(),
            },
        }
    }
}
