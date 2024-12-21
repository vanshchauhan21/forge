use std::path::PathBuf;

use forge_provider::model::{AnyMessage, Message, Request, ToolResult, ToolUse};
use forge_provider::Provider;
use forge_tool::{Prompt, Router};
use futures::future::join_all;
use futures::FutureExt;
use serde_json::Value;

use crate::completion::Completion;
use crate::error::Result;
use crate::walker::Walker;
pub struct Engine {
    tool_engine: Router,
    provider: Provider,
    walker: Walker,
}

impl Engine {
    pub fn new(key: String, cwd: PathBuf) -> Self {
        Self {
            tool_engine: Router::default(),
            provider: Provider::open_router(key, None, None),
            walker: Walker::new(cwd),
        }
    }

    pub async fn launch(&self) -> Result<()> {
        let prompt = self.ask(None).await?;
        let mut request = Request::default()
            .add_message(Message::system(include_str!("./prompts/system.md")))
            .add_message(prompt)
            .tools(self.tool_engine.list());

        loop {
            let response = self.provider.chat(request.clone()).await?;
            if !response.tool_use.is_empty() {
                let results = join_all(
                    response
                        .tool_use
                        .into_iter()
                        .map(|tool_use| self.use_tool(tool_use)),
                )
                .await;

                request = request.extend_tool_results(results);
            } else {
                let prompt = self.ask(None).await?;
                request = request.add_message(prompt);
            }
        }
    }

    async fn ask(&self, message: Option<&str>) -> Result<ResolvePrompt> {
        let project_files = self.walker.get().await?;
        let completions =
            Completion::new(project_files.iter().map(|s| format!("@{}", s)).collect());

        let input = inquire::Text::new(message.unwrap_or(""))
            .with_autocomplete(completions)
            .prompt()?;

        let prompt = Prompt::parse(input)?;

        let files = join_all(prompt.files().into_iter().map(|path| {
            tokio::fs::read_to_string(path.clone())
                .map(|result| result.map(|content| File { path, content }))
        }))
        .await;

        Ok(ResolvePrompt {
            message: prompt.message(),
            files: files.into_iter().flatten().collect(),
        })
    }

    async fn use_tool(&self, tool: ToolUse) -> ToolResult {
        let result = self
            .tool_engine
            .call(&tool.tool_id, tool.input.clone())
            .await;

        match result {
            Ok(content) => ToolResult { tool_use_id: tool.tool_use_id, content },
            Err(error) => ToolResult { tool_use_id: tool.tool_use_id, content: Value::from(error) },
        }
    }
}

pub struct ResolvePrompt {
    message: String,
    files: Vec<File>,
}

pub struct File {
    path: String,
    content: String,
}

impl From<ResolvePrompt> for AnyMessage {
    fn from(prompt: ResolvePrompt) -> Self {
        let message = format!(
            "{}\n{}",
            prompt.message,
            prompt
                .files
                .iter()
                .map(|file| format!("<file path={}>\n{}\n<file>", file.path, file.content))
                .collect::<Vec<String>>()
                .join("\n")
        );

        AnyMessage::User(Message::user(message))
    }
}
