use anyhow::Result;
use colored::Colorize;
use forge_app::Routes;
use forge_domain::{ChatRequest, ChatResponse, Command, ConversationId, ModelId, Usage, UserInput};
use tokio_stream::StreamExt;

use crate::{Console, StatusDisplay, CONSOLE};

pub struct UI {
    current_conversation_id: Option<ConversationId>,
    current_title: Option<String>,
    current_content: Option<String>,
    usage: Usage,
    api: Routes,
    console: Console,
    verbose: bool,
    exec: Option<String>,
}

impl UI {
    pub async fn new(verbose: bool, exec: Option<String>) -> Result<Self> {
        let api = Routes::init()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize API: {}", e))?;

        Ok(Self {
            current_conversation_id: None,
            current_title: None,
            current_content: None,
            usage: Default::default(),
            api,
            console: Console,
            verbose,
            exec,
        })
    }

    fn context_reset_message(&self, _: &Command) -> String {
        "All context was cleared, and we're starting fresh. Please re-add files and details so we can get started.".to_string()
            .yellow()
            .bold()
            .to_string()
    }

    pub async fn run(&mut self) -> Result<()> {
        // Get initial input from file or prompt
        let mut input = match &self.exec {
            Some(ref path) => self.console.upload(path).await?,
            None => self.console.prompt(None, None).await?,
        };

        let model = ModelId::from_env(&self.api.environment().await?);

        loop {
            match input {
                Command::End => break,
                Command::New => {
                    CONSOLE.writeln(self.context_reset_message(&input))?;
                    self.current_conversation_id = None;
                    self.current_title = None;
                    input = self.console.prompt(None, None).await?;
                    continue;
                }
                Command::Reload => {
                    CONSOLE.writeln(self.context_reset_message(&input))?;
                    self.current_conversation_id = None;
                    self.current_title = None;
                    input = match &self.exec {
                        Some(ref path) => self.console.upload(path).await?,
                        None => {
                            self.console
                                .prompt(None, self.current_content.as_deref())
                                .await?
                        }
                    };
                    continue;
                }
                Command::Info => {
                    crate::display_info(&self.api.environment().await?, &self.usage)?;
                    input = self
                        .console
                        .prompt(self.current_title().as_deref(), None)
                        .await?;
                    continue;
                }
                Command::Message(ref content) => {
                    self.current_content = Some(content.clone());
                    self.handle_message(content.clone(), &model).await?;
                    input = self
                        .console
                        .prompt(self.current_title().as_deref(), None)
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn handle_message(&mut self, content: String, model: &ModelId) -> Result<()> {
        let chat = ChatRequest {
            content,
            model: model.clone(),
            conversation_id: self.current_conversation_id,
        };

        match self.api.chat(chat).await {
            Ok(mut stream) => {
                while let Some(message) = stream.next().await {
                    match message {
                        Ok(message) => self.handle_chat_response(message)?,
                        Err(err) => {
                            CONSOLE.writeln(
                                StatusDisplay::failed(err.to_string(), self.usage.clone()).format(),
                            )?;
                        }
                    }
                }
            }
            Err(err) => {
                CONSOLE.writeln(
                    StatusDisplay::failed_with(
                        err.to_string().as_str(),
                        "Failed to establish chat stream",
                        self.usage.clone(),
                    )
                    .format(),
                )?;
            }
        }

        Ok(())
    }

    fn handle_chat_response(&mut self, message: ChatResponse) -> Result<()> {
        match message {
            ChatResponse::Text(text) => {
                CONSOLE.write(&text)?;
            }
            ChatResponse::ToolCallDetected(_) => {}
            ChatResponse::ToolCallArgPart(arg) => {
                if self.verbose {
                    CONSOLE.write(&arg)?;
                }
            }
            ChatResponse::ToolCallStart(tool_call_full) => {
                let tool_name = tool_call_full.name.as_str();
                CONSOLE.newline()?;
                CONSOLE.writeln(StatusDisplay::execute(tool_name, self.usage.clone()).format())?;

                // Convert to JSON and apply dimmed style
                let json = serde_json::to_string_pretty(&tool_call_full.arguments)
                    .unwrap_or_else(|_| "Failed to serialize arguments".to_string());

                CONSOLE.writeln(format!("{}", json.dimmed()))?;
            }
            ChatResponse::ToolCallEnd(tool_result) => {
                let tool_name = tool_result.name.as_str();
                // Always show result content for errors, or in verbose mode
                if tool_result.is_error || self.verbose {
                    CONSOLE.writeln(format!("{}", tool_result.to_string().dimmed()))?;
                }
                let status = if tool_result.is_error {
                    StatusDisplay::failed(tool_name, self.usage.clone())
                } else {
                    StatusDisplay::success(tool_name, self.usage.clone())
                };

                CONSOLE.write(status.format())?;
            }
            ChatResponse::ConversationStarted(conversation_id) => {
                self.current_conversation_id = Some(conversation_id);
            }
            ChatResponse::ModifyContext(_) => {}
            ChatResponse::Complete => {}
            ChatResponse::Error(err) => {
                CONSOLE
                    .writeln(StatusDisplay::failed(err.to_string(), self.usage.clone()).format())?;
            }
            ChatResponse::PartialTitle(_) => {}
            ChatResponse::CompleteTitle(title) => {
                self.current_title = Some(title);
            }
            ChatResponse::FinishReason(_) => {}
            ChatResponse::Usage(u) => {
                self.usage = u;
            }
        }
        Ok(())
    }

    fn current_title(&self) -> Option<String> {
        self.current_title
            .as_ref()
            .map(|title| StatusDisplay::task(title, self.usage.clone()).format())
    }
}
