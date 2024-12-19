use crate::model::State;
use crate::{error::Result, model::Event, Stream};
use forge_provider::Provider;
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
        CodeForge {
            state: Arc::new(Mutex::new(State::default())),
            tools: HashMap::new(),
            provider: Provider::open_router(key, None, None),
        }
    }

    pub fn add_tool<T: Tool + Sync + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub async fn prompt(&self, prompt: String) -> Result<Stream<Event>> {
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
