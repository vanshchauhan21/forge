use serde_json::Value;

pub struct Request {
    context: Vec<AnyMessage>,
    available_tools: Vec<Tool>,
}

pub struct System {}
pub struct User {}
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

pub struct Message<Role> {
    content: String,
    role: Role,
}

pub enum AnyMessage {
    System(Message<System>),
    User(Message<User>),
    Assistant(Message<Assistant>),
}

pub struct ToolId(String);

pub struct Tool {
    id: ToolId,
    description: String,
    input_schema: JsonSchema,
    output_schema: JsonSchema,
}

pub struct Response {
    message: Message<Assistant>,
    call_tool: Vec<CallTool>,
}

pub struct CallId(String);

pub struct CallTool {
    id: CallId,
    tool_id: ToolId,
    input: Value,
}

pub struct JsonSchema(Value);
