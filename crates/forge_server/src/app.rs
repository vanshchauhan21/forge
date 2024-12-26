use forge_prompt::Prompt;
use forge_provider::{FinishReason, Message, ModelId, Request, Response};
use forge_tool::ToolName;
use serde_json::Value;

use crate::template::MessageTemplate;
use crate::Result;
pub enum Action {
    UserChatMessage(ChatRequest),
    PromptFileLoaded(Vec<FileResponse>),
    AgentChatResponse(Response),
    ToolUseResponse(String),
}

pub struct FileResponse {
    pub path: String,
    pub content: String,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct ChatRequest {
    // Add fields as needed, for example:
    pub message: String,
    pub model: ModelId,
}

#[derive(Debug, Clone, Default)]
pub enum Command {
    #[default]
    Empty,
    Combine(Box<Command>, Box<Command>),
    LoadPromptFiles(Vec<String>),
    DispatchAgentMessage(Request),
    DispatchUserMessage(String),
    DispatchToolUse(ToolName, Value),
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

    fn when(self, condition: bool) -> Self {
        if condition {
            self
        } else {
            Command::Empty
        }
    }
}

#[derive(Debug, Default)]
pub struct App {
    context: Request,
    user_message: Option<MessageTemplate>,
    tool_use: bool,
    tool_raw_arguments: String,
    tool_name: Option<ToolName>,
}

impl App {
    fn run(mut self, action: Action) -> Result<(Self, Command)> {
        let cmd: Command = match action {
            Action::UserChatMessage(chat) => {
                let prompt = Prompt::parse(chat.message.clone())
                    .unwrap_or(Prompt::new(chat.message.clone()));

                self.context = self.context.model(chat.model.clone());
                self.user_message = Some(MessageTemplate::task(prompt.to_string()));

                Command::LoadPromptFiles(prompt.files()).when(!prompt.files().is_empty())
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
                if response.tool_use.is_empty() {
                    Command::DispatchUserMessage(response.message.content)
                } else {
                    self.tool_use = true;
                    let mut commands = Vec::new();
                    for tool in response.tool_use.into_iter() {
                        if let Some(tool) = tool.tool_name {
                            self.tool_name = Some(tool)
                        }
                        self.tool_raw_arguments.push_str(tool.input.as_str());

                        if let Some(FinishReason::ToolUse) = response.finish_reason {
                            let argument = serde_json::from_str(&self.tool_raw_arguments)?;
                            if let Some(tool_name) = self.tool_name.clone() {
                                commands.push(Command::DispatchToolUse(tool_name, argument));
                            }
                        }
                    }

                    Command::from(commands)
                }
            }
            Action::ToolUseResponse(response) => {
                self.context = self.context.add_message(Message::user(response));
                Command::DispatchAgentMessage(self.context.clone())
            }
        };
        Ok((self, cmd))
    }
}

#[cfg(test)]
mod test {}
