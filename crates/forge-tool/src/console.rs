use forge_prompt::PromptData;
use forge_tool_macros::Description;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{Description, ToolTrait};

/// Read a line from the console
#[derive(Description)]
pub(crate) struct ReadLine {
    prompt: forge_prompt::UserPrompt,
}

#[derive(JsonSchema, Serialize)]
pub struct File {
    pub path: String,
    pub content: String,
}

#[derive(JsonSchema, Serialize)]
pub struct ReadLineOutput {
    pub message: String,
    pub files: Vec<File>,
}

impl From<PromptData> for ReadLineOutput {
    fn from(value: PromptData) -> Self {
        let files = value
            .files
            .into_iter()
            .map(|file| File { content: file.content, path: file.path })
            .collect();

        Self { message: value.message, files }
    }
}

impl Default for ReadLine {
    fn default() -> Self {
        Self {
            prompt: forge_prompt::UserPrompt::new(std::env::current_dir().unwrap()),
        }
    }
}

/// Write a line to the console
#[derive(Description)]
pub(crate) struct WriteLine;

#[derive(JsonSchema, Deserialize)]
pub struct ReadLineInput {
    pub message: Option<String>,
}

#[async_trait::async_trait]
impl ToolTrait for ReadLine {
    type Input = ReadLineInput;
    type Output = ReadLineOutput;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let message = input.message;
        let prompt = self
            .prompt
            .ask(message.as_deref())
            .await
            // TODO: Can't return strings over here
            .map_err(|e| e.to_string())?;

        Ok(prompt.into())
    }
}

#[derive(JsonSchema, Deserialize)]
pub struct WriteLineInput {
    pub message: String,
}

#[async_trait::async_trait]
impl ToolTrait for WriteLine {
    type Input = WriteLineInput;
    type Output = ();

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        println!("{}", input.message);
        Ok(())
    }
}
