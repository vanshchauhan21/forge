pub mod error;
mod model;

use error::Result;
use forge_tool::{JsonRpcRequest, JsonRpcResponse, Tool};
use model::State;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;

#[derive(Default)]
pub struct CodeForge {
    state: Arc<Mutex<State>>,
    tools: HashMap<String, Box<dyn Tool<Input = JsonRpcRequest, Output = JsonRpcResponse>>>,
}

impl CodeForge {
    pub fn add_tool<T: Tool + Sync + 'static>(&mut self, tool: T)
    where
        T::Input: TryFrom<JsonRpcRequest, Error = forge_tool::error::Error>,
        T::Output: TryInto<JsonRpcResponse, Error = forge_tool::error::Error>,
    {
        self.tools
            .insert(tool.name().to_string(), Box::new(tool.into_dyn()));
    }

    pub async fn prompt(&self, prompt: &str) -> Result<Stream<Event>> {
        todo!()
    }

    pub async fn model(&self, model: &str) -> Result<Stream<Event>> {
        todo!()
    }

    /// Returns an autocomplete for a prompt containing '@'
    pub async fn files(&self) -> Result<Vec<String>> {
        todo!()
    }

    pub async fn models(&self) -> Result<Vec<String>> {
        todo!()
    }
}

pub enum Event {}
