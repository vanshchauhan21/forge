use derive_setters::Setters;
use forge_tool::ToolDefinition;
use serde::{Deserialize, Serialize};

use super::CompletionMessage;

/// Represents a request being made to the LLM provider. By default the request
/// is created with assuming the model supports use of external tools.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, Setters)]
pub struct Request {
    pub messages: Vec<CompletionMessage>,
    pub model: ModelId,
    pub tools: Vec<ToolDefinition>,
}

impl Request {
    pub fn new(id: ModelId) -> Self {
        Request { messages: vec![], tools: vec![], model: id }
    }

    pub fn add_tool(mut self, tool: impl Into<ToolDefinition>) -> Self {
        let tool = tool;
        let tool: ToolDefinition = tool.into();
        self.tools.push(tool);

        self
    }

    pub fn add_message(mut self, content: impl Into<CompletionMessage>) -> Self {
        self.messages.push(content.into());
        self
    }

    pub fn extend_tools(mut self, tools: Vec<impl Into<ToolDefinition>>) -> Self {
        self.tools.extend(tools.into_iter().map(Into::into));
        self
    }

    pub fn extend_messages(mut self, messages: Vec<impl Into<CompletionMessage>>) -> Self {
        self.messages.extend(messages.into_iter().map(Into::into));
        self
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Setters)]
pub struct Model {
    pub id: ModelId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ModelId(String);

impl Default for ModelId {
    fn default() -> Self {
        ModelId("openai/gpt-3.5-turbo".to_string())
    }
}
