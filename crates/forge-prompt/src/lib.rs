mod completion;
mod error;
mod walker;
use std::path::PathBuf;
mod prompt;
use colorize::AnsiColor;
use completion::Completion;
pub use error::*;
use futures::future::join_all;
use futures::FutureExt;
use inquire::ui::{RenderConfig, Styled};
use prompt::Prompt;
use serde::{Deserialize, Serialize};
use walker::Walker;

pub struct UserPrompt {
    walker: Walker,
}

#[derive(Serialize, Deserialize)]
pub struct PromptData {
    pub message: String,
    pub files: Vec<File>,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub path: String,
    pub content: String,
}

impl UserPrompt {
    pub fn new(cwd: PathBuf) -> Self {
        Self { walker: Walker::new(cwd) }
    }

    pub async fn ask(&self, message: Option<&str>) -> Result<PromptData> {
        let suggestions = self.walker.get()?;
        let completions = Completion::new(suggestions.iter().map(|s| format!("@{}", s)).collect());
        let dot = "◉".cyan();
        let config = RenderConfig {
            prompt_prefix: Styled::new("○"),
            answered_prompt_prefix: Styled::new(dot.as_str()),
            ..RenderConfig::default()
        };
        let input = inquire::Text::new(message.unwrap_or(""))
            .with_autocomplete(completions)
            .with_render_config(config)
            .prompt()?;

        let prompt = Prompt::parse(input).map_err(Error::Parse)?;

        let files = join_all(prompt.files().into_iter().map(|path| {
            tokio::fs::read_to_string(path.clone())
                .map(|result| result.map(|content| File { path, content }))
        }))
        .await;

        Ok(PromptData {
            message: prompt.message(),
            files: files.into_iter().flatten().collect(),
        })
    }
}
