use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use super::{ContextMessage, ModelId, Role};
use crate::ToolDefinition;

/// Represents a request being made to the LLM provider. By default the request
/// is created with assuming the model supports use of external tools.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, Setters)]
pub struct Context {
    pub messages: Vec<ContextMessage>,
    pub model: ModelId,
    pub tools: Vec<ToolDefinition>,
}

impl Context {
    pub fn new(id: ModelId) -> Self {
        Context { messages: vec![], tools: vec![], model: id }
    }

    pub fn add_tool(mut self, tool: impl Into<ToolDefinition>) -> Self {
        let tool = tool;
        let tool: ToolDefinition = tool.into();
        self.tools.push(tool);

        self
    }

    pub fn add_message(mut self, content: impl Into<ContextMessage>) -> Self {
        self.messages.push(content.into());
        self
    }

    pub fn extend_tools(mut self, tools: Vec<impl Into<ToolDefinition>>) -> Self {
        self.tools.extend(tools.into_iter().map(Into::into));
        self
    }

    pub fn extend_messages(mut self, messages: Vec<impl Into<ContextMessage>>) -> Self {
        self.messages.extend(messages.into_iter().map(Into::into));
        self
    }

    /// Updates the set system message
    pub fn set_system_message(mut self, content: impl Into<String>) -> Self {
        if self.messages.is_empty() {
            self.add_message(ContextMessage::system(content.into()))
        } else {
            if let Some(ContextMessage::ContentMessage(content_message)) = self.messages.get_mut(0)
            {
                if content_message.role == Role::System {
                    content_message.content = content.into();
                } else {
                    self.messages
                        .insert(0, ContextMessage::system(content.into()));
                }
            }

            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_override_system_message() {
        let request = Context::new(ModelId::default())
            .add_message(ContextMessage::system("Initial system message"))
            .set_system_message("Updated system message");

        assert_eq!(
            request.messages[0],
            ContextMessage::system("Updated system message")
        );
    }

    #[test]
    fn test_set_system_message() {
        let request = Context::new(ModelId::default()).set_system_message("A system message");

        assert_eq!(
            request.messages[0],
            ContextMessage::system("A system message")
        );
    }

    #[test]
    fn test_insert_system_message() {
        let request = Context::new(ModelId::default())
            .add_message(ContextMessage::user("Do something"))
            .set_system_message("A system message");

        assert_eq!(
            request.messages[0],
            ContextMessage::system("A system message")
        );
    }
}
