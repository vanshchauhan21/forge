use derive_setters::Setters;
use serde_json::Value;

#[derive(Setters, Debug, Clone)]
pub struct Request {
    pub context: Vec<AnyMessage>,
    pub available_tools: Vec<Tool>,
}

#[derive(Setters, Debug, Clone)]
pub struct System {}
#[derive(Setters, Debug, Clone)]
pub struct User {}
#[derive(Setters, Debug, Clone)]
pub struct Assistant {}

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

#[derive(Setters, Debug, Clone)]
pub struct Message<Role> {
    content: String,
    role: Role,
}

#[derive(Debug, Clone)]
pub enum AnyMessage {
    System(Message<System>),
    User(Message<User>),
    Assistant(Message<Assistant>),
}

#[derive(Debug, Clone)]
pub struct ToolId(String);

#[derive(Setters, Debug, Clone)]
pub struct Tool {
    pub id: ToolId,
    pub description: String,
    pub input_schema: JsonSchema,
    pub output_schema: JsonSchema,
}

#[derive(Setters, Debug, Clone)]
pub struct Response {
    pub message: Message<Assistant>,
    pub call_tool: Vec<CallTool>,
}

#[derive(Debug, Clone)]
pub struct CallId(String);

#[derive(Setters, Debug, Clone)]
pub struct CallTool {
    pub id: CallId,
    pub tool_id: ToolId,
    pub input: Value,
}

#[derive(Debug, Clone)]
pub struct JsonSchema(Value);
