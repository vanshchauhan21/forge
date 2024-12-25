use forge_prompt::Prompt;
use forge_provider::{ModelId, Request};
use forge_tool::ToolName;
use serde_json::Value;

use crate::template::PromptTemplate;
use crate::Result;
pub enum Action {
    ChatRequest(ChatRequest),
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
    ChatResponse(ChatResponse),
    LoadPromptFiles(Vec<String>),
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

#[derive(Debug, Default)]
pub struct App {
    request: Request,
}

impl App {
    fn run(&mut self, action: &Action) -> Result<Command> {
        match action {
            Action::ChatRequest(chat) => {
                let prompt = Prompt::parse(chat.message.clone())
                    .unwrap_or(Prompt::new(chat.message.clone()));
                let mut message = PromptTemplate::task(prompt.to_string());

                self.request = self.request.clone().add_message(message);

                Ok(Command::LoadPromptFiles(prompt.files()))
            }
        }
    }
}

#[cfg(test)]
mod test {}
