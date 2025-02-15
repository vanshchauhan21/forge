use std::sync::Arc;

use anyhow::Result;
use colored::Colorize;
use forge_api::{
    AgentMessage, ChatRequest, ChatResponse, ConversationId, Model, Usage, Workflow, API,
};
use forge_display::TitleFormat;
use forge_tracker::EventKind;
use lazy_static::lazy_static;
use tokio_stream::StreamExt;

use crate::cli::Cli;
use crate::console::CONSOLE;
use crate::info::Info;
use crate::input::{Console, PromptInput};
use crate::model::{Command, UserInput};
use crate::{banner, log};

lazy_static! {
    pub static ref TRACKER: forge_tracker::Tracker = forge_tracker::Tracker::default();
}

#[derive(Default)]
struct UIState {
    current_title: Option<String>,
    conversation_id: Option<ConversationId>,
    usage: Usage,
}

impl From<&UIState> for PromptInput {
    fn from(state: &UIState) -> Self {
        PromptInput::Update {
            title: state.current_title.clone(),
            usage: Some(state.usage.clone()),
        }
    }
}

pub struct UI<F> {
    state: UIState,
    api: Arc<F>,
    console: Console,
    cli: Cli,
    models: Option<Vec<Model>>,
    #[allow(dead_code)] // The guard is kept alive by being held in the struct
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

impl<F: API> UI<F> {
    pub fn init(cli: Cli, api: Arc<F>) -> Result<Self> {
        // Parse CLI arguments first to get flags

        let env = api.environment();
        let guard = log::init_tracing(env.clone())?;

        Ok(Self {
            state: Default::default(),
            api,
            console: Console::new(env),
            cli,
            models: None,
            _guard: guard,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Handle direct prompt if provided
        let prompt = self.cli.prompt.clone();
        if let Some(prompt) = prompt {
            self.chat(prompt).await?;
            return Ok(());
        }

        // Display the banner in dimmed colors since we're in interactive mode
        banner::display()?;

        // Get initial input from file or prompt
        let mut input = match &self.cli.command {
            Some(path) => self.console.upload(path).await?,
            None => self.console.prompt(None).await?,
        };

        loop {
            match input {
                Command::New => {
                    banner::display()?;
                    self.state = Default::default();
                    input = self.console.prompt(None).await?;

                    continue;
                }
                Command::Info => {
                    let info =
                        Info::from(&self.api.environment()).extend(Info::from(&self.state.usage));

                    CONSOLE.writeln(info.to_string())?;

                    let prompt_input = Some((&self.state).into());
                    input = self.console.prompt(prompt_input).await?;
                    continue;
                }
                Command::Message(ref content) => {
                    if let Err(err) = self.chat(content.clone()).await {
                        CONSOLE.writeln(
                            TitleFormat::failed(format!("{:?}", err))
                                .sub_title(self.state.usage.to_string())
                                .format(),
                        )?;
                    }
                    let prompt_input = Some((&self.state).into());
                    input = self.console.prompt(prompt_input).await?;
                }
                Command::Exit => {
                    break;
                }
                Command::Models => {
                    let models = if let Some(models) = self.models.as_ref() {
                        models
                    } else {
                        let models = self.api.models().await?;
                        self.models = Some(models);
                        self.models.as_ref().unwrap()
                    };
                    let info: Info = models.as_slice().into();
                    CONSOLE.writeln(info.to_string())?;

                    input = self.console.prompt(None).await?;
                }
            }
        }

        Ok(())
    }

    async fn init_workflow(&self) -> anyhow::Result<Workflow> {
        match self.cli.workflow {
            Some(ref path) => self.api.load(path).await,
            None => Ok(include_str!("../../../templates/workflows/default.toml").parse()?),
        }
    }

    async fn chat(&mut self, content: String) -> Result<()> {
        let conversation_id = match self.state.conversation_id {
            Some(ref id) => id.clone(),
            None => {
                let conversation_id = self.api.init(self.init_workflow().await?).await?;
                self.state.conversation_id = Some(conversation_id.clone());

                conversation_id
            }
        };

        let chat = ChatRequest { content: content.clone(), conversation_id };

        tokio::spawn({
            let content = content.clone();
            async move {
                let _ = TRACKER.dispatch(EventKind::Prompt(content)).await;
            }
        });
        match self.api.chat(chat).await {
            Ok(mut stream) => self.handle_chat_stream(&mut stream).await,
            Err(err) => Err(err),
        }
    }

    async fn handle_chat_stream(
        &mut self,
        stream: &mut (impl StreamExt<Item = Result<AgentMessage<ChatResponse>>> + Unpin),
    ) -> Result<()> {
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    return Ok(());
                }
                maybe_message = stream.next() => {
                    match maybe_message {
                        Some(Ok(message)) => self.handle_chat_response(message)?,
                        Some(Err(err)) => {
                            return Err(err);
                        }
                        None => return Ok(()),
                    }
                }
            }
        }
    }

    fn handle_chat_response(&mut self, message: AgentMessage<ChatResponse>) -> Result<()> {
        match message.message {
            ChatResponse::Text(text) => {
                if message.agent.as_str() == "developer" {
                    CONSOLE.write(&text)?;
                }
            }
            ChatResponse::ToolCallStart(_) => {
                CONSOLE.newline()?;
                CONSOLE.newline()?;
            }
            ChatResponse::ToolCallEnd(tool_result) => {
                if !self.cli.verbose {
                    return Ok(());
                }

                let tool_name = tool_result.name.as_str();

                CONSOLE.writeln(format!("{}", tool_result.content.dimmed()))?;

                if tool_result.is_error {
                    CONSOLE.writeln(
                        TitleFormat::failed(tool_name)
                            .sub_title(self.state.usage.to_string())
                            .format(),
                    )?;
                } else {
                    CONSOLE.writeln(
                        TitleFormat::success(tool_name)
                            .sub_title(self.state.usage.to_string())
                            .format(),
                    )?;
                }
            }
            ChatResponse::Custom(event) => {
                if event.name == "title" {
                    self.state.current_title = Some(event.value);
                }
            }
            ChatResponse::Usage(u) => {
                self.state.usage = u;
            }
        }
        Ok(())
    }
}
