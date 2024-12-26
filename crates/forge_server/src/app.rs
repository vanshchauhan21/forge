use derive_more::derive::From;
use derive_setters::Setters;
use forge_prompt::Prompt;
use forge_provider::{FinishReason, Message, ModelId, Request, Response, ToolResult};
use forge_tool::ToolName;
use serde::Serialize;
use serde_json::Value;

use crate::runtime::Application;
use crate::template::MessageTemplate;
use crate::Result;

#[derive(Debug, From)]
pub enum Action {
    UserChatMessage(ChatRequest),
    PromptFileLoaded(Vec<FileResponse>),
    AgentChatResponse(Response),
    ToolUseResponse(ToolResult),
}

#[derive(Debug, Clone)]
pub struct FileResponse {
    pub path: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize, Clone, Setters)]
pub struct ChatRequest {
    pub message: String,
    pub model: ModelId,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Command {
    #[default]
    Empty,
    Combine(Box<Command>, Box<Command>),
    LoadPromptFiles(Vec<String>),
    DispatchAgentMessage(Request),
    DispatchUserMessage(String),
    DispatchToolUse {
        tool_name: ToolName,
        arguments: Value,
    },
}

impl<T> From<T> for Command
where
    T: IntoIterator<Item = Command>,
{
    fn from(value: T) -> Self {
        let mut command = Command::default();
        for cmd in value.into_iter() {
            command = command.and_then(cmd);
        }

        command
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub enum ChatResponse {
    Text(String),
    ToolUseStart(ToolName),
    ToolUseEnd(String, Value),
    Complete,
    Fail(String),
}

impl Command {
    fn and_then(self, second: Command) -> Self {
        Self::Combine(Box::new(self), Box::new(second))
    }
}

#[derive(Default, Debug, Clone, Serialize, Setters)]
#[setters(strip_option)]
pub struct App {
    pub user_message: Option<MessageTemplate>,
    pub assistant_buffer: String,
    pub tool_use: bool,
    pub tool_raw_arguments: String,
    pub tool_name: Option<ToolName>,
    pub files: Vec<String>,

    // Keep context at the end so that debugging the Serialized format is easier
    pub context: Request,
}

impl App {
    pub fn new(context: Request) -> Self {
        Self {
            context,
            user_message: None,
            tool_use: false,
            tool_raw_arguments: "".to_string(),
            tool_name: None,
            assistant_buffer: "".to_string(),
            files: Vec::new(),
        }
    }
}

impl Application for App {
    type Action = Action;
    type Error = crate::Error;
    type Command = Command;

    fn update(mut self, action: Action) -> Result<(Self, Command)> {
        let cmd: Command = match action {
            Action::UserChatMessage(chat) => {
                let prompt = Prompt::parse(chat.message.clone())
                    .unwrap_or(Prompt::new(chat.message.clone()));

                self.context = self.context.model(chat.model.clone());
                self.user_message = Some(MessageTemplate::task(prompt.to_string()));

                if prompt.files().is_empty() {
                    self.context = self.context.add_message(Message::user(chat.message));
                    Command::DispatchAgentMessage(self.context.clone())
                } else {
                    Command::LoadPromptFiles(prompt.files())
                }
            }
            Action::PromptFileLoaded(files) => {
                if let Some(message) = self.user_message.clone() {
                    for fr in files.into_iter() {
                        self.context = self.context.add_message(
                            message
                                .clone()
                                .append(MessageTemplate::file(fr.path, fr.content)),
                        );
                    }

                    Command::DispatchAgentMessage(self.context.clone())
                } else {
                    Command::default()
                }
            }
            Action::AgentChatResponse(response) => {
                self.assistant_buffer
                    .push_str(response.message.content.as_str());
                if response.finish_reason.is_some() {
                    self.context = self
                        .context
                        .add_message(Message::assistant(self.assistant_buffer.clone()));
                    self.assistant_buffer.clear();
                }

                self.tool_use = true;
                let mut commands = Vec::new();
                for tool in response.tool_use.into_iter() {
                    if let Some(tool) = tool.tool_name {
                        self.tool_name = Some(tool)
                    }
                    self.tool_raw_arguments.push_str(tool.input.as_str());
                }

                if let Some(FinishReason::ToolUse) = response.finish_reason {
                    self.tool_use = false;
                    let arguments = serde_json::from_str(&self.tool_raw_arguments)?;
                    if let Some(tool_name) = self.tool_name.clone() {
                        commands.push(Command::DispatchToolUse { tool_name, arguments });
                    }
                    // since tools is used, clear the tool_raw_arguments.
                    self.tool_raw_arguments.clear();
                }

                Command::DispatchUserMessage(response.message.content).and_then(commands.into())
            }
            Action::ToolUseResponse(response) => {
                let message = if response.is_error {
                    "An error occurred while processing the tool.".to_string()
                } else {
                    "Tool executed successfully.".to_string()
                };

                self.context = self
                    .context
                    .add_message(Message::user(message))
                    .add_tool_result(response);

                Command::DispatchAgentMessage(self.context.clone())
            }
        };
        Ok((self, cmd))
    }
}

#[cfg(test)]
mod tests {
    use forge_provider::Message;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::template::Tag;

    #[test]
    fn test_user_chat_message_action() {
        let app = App::default();

        let chat_request = ChatRequest {
            message: "Hello, world!".to_string(),
            model: ModelId::default(),
        };

        let action = Action::UserChatMessage(chat_request.clone());
        let (updated_app, command) = app.update(action).unwrap();

        assert_eq!(&updated_app.context.model, &ModelId::default());
        assert_eq!(
            command,
            Command::DispatchAgentMessage(updated_app.context.clone())
        );
    }

    #[test]
    fn test_prompt_file_loaded_action() {
        let app = App::default().user_message(MessageTemplate::new(
            Tag { name: "test".to_string(), attributes: vec![] },
            "Test message".to_string(),
        ));

        let files = vec![FileResponse {
            path: "test_path.txt".to_string(),
            content: "Test content".to_string(),
        }];

        let action = Action::PromptFileLoaded(files.clone());
        let (updated_app, command) = app.update(action).unwrap();

        assert_eq!(
            command,
            Command::DispatchAgentMessage(updated_app.context.clone())
        );

        assert!(updated_app.context.messages[0]
            .content()
            .contains(&files[0].path));
        assert!(updated_app.context.messages[0]
            .content()
            .contains(&files[0].content));
    }

    #[test]
    fn test_agent_chat_response_action_with_tool_use() {
        let app = App::default();

        let response = Response {
            message: Message::assistant("Tool response"),
            tool_use: vec![forge_provider::ToolUse {
                tool_use_id: None,
                tool_name: Some(ToolName::from("test_tool")),
                input: r#"{"key": "value"}"#.to_string(),
            }],
            finish_reason: Some(FinishReason::ToolUse),
        };

        let action = Action::AgentChatResponse(response);
        let (_, command) = app.update(action).unwrap();

        match command {
            Command::Combine(left, right) => {
                assert_eq!(
                    *left,
                    Command::DispatchUserMessage("Tool response".to_string())
                );
                match *right {
                    Command::Combine(_, right_inner) => {
                        let expected_tool_name = ToolName::from("test_tool");
                        let expected_tool_args: Value =
                            serde_json::from_str(r#"{"key": "value"}"#).unwrap();
                        assert_eq!(
                            *right_inner,
                            Command::DispatchToolUse {
                                tool_name: expected_tool_name,
                                arguments: expected_tool_args
                            }
                        );
                    }
                    _ => panic!("Expected nested Combine command"),
                }
            }
            _ => panic!("Expected Command::Combine"),
        }
    }

    #[test]
    fn test_tool_use_response_action() {
        let app = App::default();

        let tool_response = json!({
            "key": "value",
            "nested": {
                "key": "value"
            }
        });
        let action = Action::ToolUseResponse(ToolResult {
            tool_use_id: None,
            tool_name: ToolName::from("test_tool"),
            content: tool_response.clone(),
            is_error: false,
        });

        let (updated_app, command) = app.update(action).unwrap();

        assert_eq!(
            command,
            Command::DispatchAgentMessage(updated_app.context.clone())
        );
        assert!(updated_app.context.messages[0]
            .content()
            .contains("Tool executed successfully."));
    }

    #[test]
    fn test_combine_commands() {
        let cmd1 = Command::DispatchUserMessage("First command".to_string());
        let cmd2 = Command::DispatchUserMessage("Second command".to_string());

        let combined = cmd1.and_then(cmd2);

        match combined {
            Command::Combine(left, right) => {
                assert_eq!(
                    *left,
                    Command::DispatchUserMessage("First command".to_string())
                );
                assert_eq!(
                    *right,
                    Command::DispatchUserMessage("Second command".to_string())
                );
            }
            _ => panic!("Expected Command::Combine"),
        }
    }

    #[test]
    fn test_use_tool_when_finish_reason_present() {
        let app = App::default();
        let response = Response {
            message: Message::assistant("Tool response"),
            tool_use: vec![forge_provider::ToolUse {
                tool_use_id: None,
                tool_name: Some(ToolName::from("fs_list")),
                input: r#"{"path": "."}"#.to_string(),
            }],
            finish_reason: Some(FinishReason::ToolUse),
        };

        let action = Action::AgentChatResponse(response);
        let (app, command) = app.update(action).unwrap();

        assert!(app.tool_raw_arguments.is_empty());
        match command {
            Command::Combine(left, right) => {
                assert_eq!(
                    *left,
                    Command::DispatchUserMessage("Tool response".to_string())
                );
                match *right {
                    Command::Combine(_, right_inner) => {
                        let expected_tool_name = ToolName::from("fs_list");
                        let expected_tool_args: Value =
                            serde_json::from_str(r#"{"path": "."}"#).unwrap();
                        assert_eq!(
                            *right_inner,
                            Command::DispatchToolUse {
                                tool_name: expected_tool_name,
                                arguments: expected_tool_args
                            }
                        );
                    }
                    _ => panic!("Expected nested Combine command"),
                }
            }
            _ => panic!("Expected Command::Combine"),
        }
    }

    #[test]
    fn test_should_not_use_tool_when_finish_reason_not_present() {
        let app = App::default();
        let response = Response {
            message: Message::assistant("Tool response"),
            tool_use: vec![forge_provider::ToolUse {
                tool_use_id: None,
                tool_name: Some(ToolName::from("fs_list")),
                input: r#"{"path": "."}"#.to_string(),
            }],
            finish_reason: None,
        };

        let action = Action::AgentChatResponse(response);
        let (app, command) = app.update(action).unwrap();
        assert!(!app.tool_raw_arguments.is_empty());
        match command {
            Command::Combine(left, right) => {
                assert_eq!(
                    *left,
                    Command::DispatchUserMessage("Tool response".to_string())
                );
                assert_eq!(*right, Command::Empty);
            }
            _ => panic!("Expected Command::Combine"),
        }
    }
}
