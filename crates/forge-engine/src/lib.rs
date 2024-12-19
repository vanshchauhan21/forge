pub mod error;
mod model;

use error::Result;
use forge_provider::Provider;
use forge_tool::{JsonRpcRequest, JsonRpcResponse, Tool};
use model::State;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio_stream::StreamExt;

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;

pub struct CodeForge {
    state: Arc<Mutex<State>>,
    tools: HashMap<String, Box<dyn Tool<Input = JsonRpcRequest, Output = JsonRpcResponse>>>,
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

    pub fn add_tool<T: Tool + Sync + 'static>(&mut self, tool: T)
    where
        T::Input: TryFrom<JsonRpcRequest, Error = forge_tool::error::Error>,
        T::Output: TryInto<JsonRpcResponse, Error = forge_tool::error::Error>,
    {
        self.tools
            .insert(tool.name().to_string(), Box::new(tool.into_dyn()));
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

pub enum Event {
    Inquire(Option<String>),
    Text(String),
    Error(String),
    End,
}
