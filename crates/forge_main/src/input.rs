use std::path::PathBuf;

use async_trait::async_trait;
use forge_domain::{Command, Usage, UserInput};
use tokio::fs;

use crate::console::CONSOLE;
use crate::editor::{ForgeEditor, ReadResult};
use crate::prompt::ForgePrompt;
use crate::StatusDisplay;

/// Console implementation for handling user input via command line.
#[derive(Debug, Default)]
pub struct Console {
    cwd: PathBuf,
}

impl Console {
    /// Creates a new instance of `Console`.
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }
}

#[async_trait]
impl UserInput for Console {
    type PromptInput = PromptInput;
    async fn upload<P: Into<PathBuf> + Send>(&self, path: P) -> anyhow::Result<Command> {
        let path = path.into();
        let content = fs::read_to_string(&path).await?.trim().to_string();

        CONSOLE.writeln(content.clone())?;
        Ok(Command::Message(content))
    }

    async fn prompt(&self, input: Option<Self::PromptInput>) -> anyhow::Result<Command> {
        CONSOLE.writeln("")?;
        let mut engine = ForgeEditor::start(self.cwd.clone());
        let prompt: ForgePrompt = input.map(Into::into).unwrap_or_default();

        loop {
            let result = engine.prompt(&prompt);
            match result {
                Ok(ReadResult::Continue) => continue,
                Ok(ReadResult::Exit) => return Ok(Command::Exit),
                Ok(ReadResult::Success(text)) => match Command::parse(&text) {
                    Ok(input) => return Ok(input),
                    Err(e) => {
                        CONSOLE.writeln(
                            StatusDisplay::failed(e.to_string(), Usage::default()).format(),
                        )?;
                    }
                },
                Err(e) => {
                    CONSOLE
                        .writeln(StatusDisplay::failed(e.to_string(), Usage::default()).format())?;
                }
            }
        }
    }
}

pub enum PromptInput {
    Update {
        title: Option<String>,
        usage: Option<Usage>,
    },
}

impl From<PromptInput> for ForgePrompt {
    fn from(input: PromptInput) -> Self {
        match input {
            PromptInput::Update { title, usage } => {
                let mut prompt = ForgePrompt::default();
                if let Some(title) = title {
                    prompt.title(title);
                }
                if let Some(usage) = usage {
                    prompt.usage(usage);
                }
                prompt
            }
        }
    }
}
