pub mod error;
mod state;
mod tool;
use error::Result;
use serde_json::Value;
use state::State;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tool::SerdeTool;
pub use tool::Tool;

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;

#[derive(Default)]
pub struct CodeForge {
    state: Arc<Mutex<State>>,
    tools: HashMap<String, Box<dyn Tool<Input = Value, Output = Value>>>,
}

impl CodeForge {
    pub fn add_tool<T: Tool + Sync + 'static>(&mut self, tool: T)
    where
        T::Input: serde::de::DeserializeOwned,
        T::Output: serde::Serialize,
    {
        self.tools
            .insert(tool.name().to_string(), Box::new(SerdeTool(tool)));
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
