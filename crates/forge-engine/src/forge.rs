use crate::model::State;
use crate::{error::Result, model::Event};
use forge_provider::{Provider, Stream};
use forge_tool::Tool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio_stream::StreamExt;

pub struct CodeForge {
    state: Arc<Mutex<State>>,
    tools: HashMap<String, Box<dyn Tool>>,
    provider: Provider,
}

impl CodeForge {
    pub fn new(key: String) -> Self {
        // Add initial set of tools
        let tools: HashMap<String, Box<dyn Tool>> = vec![
            Box::new(forge_tool::FS) as Box<dyn Tool>,
            Box::new(forge_tool::Think::default()) as Box<dyn Tool>,
        ]
        .into_iter()
        .map(|tool| (tool.name().to_string(), tool))
        .collect();

        CodeForge {
            state: Arc::new(Mutex::new(State::default())),
            // TODO: add fs and think
            tools,

            // TODO: make the provider configurable
            provider: Provider::open_router(key, None, None),
        }
    }

    pub fn add_tool<T: Tool + Sync + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub async fn prompt(&self, prompt: String) -> Result<Stream<Event>> {
        // - Create Request, update context
        //   -  Add System Message
        //   -  Add Add all tools
        //   -  Add User Message
        //   -  Add Context Files
        // - Send message to LLM and await response #001
        // - On Response, dispatch event
        // - Check response has tool_use
        // - Execute tool
        // - Dispatch Event
        // - Add tool response to context
        // - Goto #001

        let stream = self.provider.prompt(prompt).await?;
        Ok(Box::new(stream.map(|message| match message {
            Ok(message) => Event::Text(message),
            Err(error) => Event::Error(format!("{}", error)),
        })))
    }

    pub fn model(self, model: String) -> Self {
        // TODO: update the provider to use the passed model
        self
    }

    /// Returns an autocomplete for a prompt containing '@'
    pub async fn files(&self) -> Result<Vec<String>> {
        todo!()
    }

    pub async fn models(&self) -> Result<Vec<String>> {
        Ok(self.provider.models().await?)
    }
}
