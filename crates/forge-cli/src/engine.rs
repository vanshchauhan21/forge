use std::path::PathBuf;

use forge_provider::model::{Message, Request, ToolResult, ToolUse};
use forge_provider::Provider;
use forge_tool::Router;
use futures::future::join_all;
use serde_json::Value;
use tracing::debug;

use crate::cli::Cli;
use crate::error::Result;
use crate::tui::Tui;

pub struct Engine {
    tool_engine: Router,
    provider: Provider,
    tui: Tui,
}

impl Engine {
    pub fn new(cli: Cli, cwd: PathBuf) -> Self {
        Self {
            tool_engine: Router::default(),
            provider: Provider::open_router(cli.key, cli.model, cli.base_url),
            tui: Tui::new(cwd),
        }
    }

    pub async fn launch(&self) -> Result<()> {
        let prompt = self.tui.ask(None).await?;
        let mut last_message = prompt.message.clone();
        let mut request = Request::default()
            .add_message(Message::system(include_str!("./prompts/system.md")))
            .add_message(Message::try_from(prompt)?)
            .tools(self.tool_engine.list());

        loop {
            let response = self
                .tui
                .task(
                    last_message.clone().as_str(),
                    self.provider.chat(request.clone()),
                )
                .await?;

            self.tui.item(response.message.content.as_str());

            if !response.tool_use.is_empty() {
                debug!(
                    "Tool use detected: {:?} /n items: {}",
                    response.tool_use,
                    response.tool_use.len()
                );
                let results = join_all(
                    response
                        .tool_use
                        .into_iter()
                        .map(|tool_use| self.use_tool(tool_use)),
                )
                .await;

                debug!("Tool results: {:?} /n items: {}", results, results.len());

                request = request.extend_tool_results(results);
            } else {
                let prompt = self.tui.ask(None).await?;
                last_message = prompt.message.clone();
                request = request.add_message(Message::try_from(prompt)?);
            }
        }
    }

    async fn use_tool(&self, tool: ToolUse) -> ToolResult {
        let engine = &self.tool_engine;

        self.tui
            .task(
                format!(
                    "{} {}",
                    tool.tool_id.clone().as_str(),
                    serde_json::to_string(&tool.input).unwrap().as_str()
                )
                .as_str(),
                async move {
                    let result = engine.call(&tool.tool_id, tool.input.clone()).await;

                    match result {
                        Ok(content) => {
                            ToolResult { tool_use_id: tool.tool_use_id, content: content.clone() }
                        }
                        Err(error) => ToolResult {
                            tool_use_id: tool.tool_use_id,
                            content: Value::from(error.to_owned()),
                        },
                    }
                },
            )
            .await
    }
}
