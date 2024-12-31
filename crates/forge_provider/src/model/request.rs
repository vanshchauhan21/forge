use derive_setters::Setters;
use forge_tool::ToolDefinition;
use serde::{Deserialize, Serialize};

use super::{CompletionMessage, Role};

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

    /// Updates the set system message
    pub fn set_system_message(mut self, content: impl Into<String>) -> Self {
        for message in self.messages.iter_mut() {
            if let CompletionMessage::ContentMessage(content_message) = message {
                if content_message.role == Role::System {
                    content_message.content = content.into();
                    break;
                }
            }
        }

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

impl ModelId {
    pub fn tool_supported(&self) -> bool {
        // This list is created by querying
        // [open router](https://openrouter.ai/api/v1/models?supported_parameters=tools)
        [
            "ai21/jamba",
            "amazon/nova",
            "anthropic/claude",
            "cohere/command-r",
            "deepseek/deepseek-chat",
            "google/gemini",
            "meta-llama/llama",
            "microsoft/phi",
            "mistralai/codestral-mamba",
            "mistralai/ministral",
            "mistralai/mistral",
            "mistralai/mixtral",
            "mistralai/pixtral",
            "nousresearch/hermes",
            "nvidia/llama",
            "openai/gpt",
            "openai/o1",
            "qwen/qwen",
            "x-ai/grok",
        ]
        .iter()
        .any(|prefix| self.0.starts_with(prefix))
    }
}

impl Default for ModelId {
    fn default() -> Self {
        ModelId("openai/gpt-3.5-turbo".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_system_message() {
        let request = Request::new(ModelId::default())
            .add_message(CompletionMessage::system("Initial system message"))
            .set_system_message("Updated system message");

        assert_eq!(
            request.messages[0],
            CompletionMessage::system("Updated system message")
        );
    }
}
