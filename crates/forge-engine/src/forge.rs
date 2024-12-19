use crate::model::{Context, Message, State};
use crate::{error::Result, model::Event};
use forge_provider::{Provider, Stream};
use forge_tool::Tool;
use std::rc::Rc;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio_stream::StreamExt;

pub struct CodeForge {
    state: Arc<Mutex<State>>,
    tools: Vec<Rc<dyn Tool>>,
    provider: Provider,
}

pub struct Prompt {
    message: String,
    files: Vec<String>,
}

impl CodeForge {
    pub fn new(key: String) -> Self {
        // Add initial set of tools
        let tools = vec![
            Rc::new(forge_tool::FS) as Rc<dyn Tool>,
            Rc::new(forge_tool::Think::default()) as Rc<dyn Tool>,
        ];

        CodeForge {
            state: Arc::new(Mutex::new(State::default())),
            // TODO: add fs and think
            tools,

            // TODO: make the provider configurable
            provider: Provider::open_router(key, None, None),
        }
    }

    pub fn add_tool<T: Tool + Sync + 'static>(&mut self, tool: T) {
        self.tools.push(Rc::new(tool));
    }

    pub async fn prompt(&self, prompt: Prompt) -> Result<Stream<Event>> {
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

        let context = Context::new(Message::system(include_str!("./prompt.md").to_string()))
            .tools(self.tools.clone())
            .add_message(Message::user(prompt.message))
            .files(prompt.files);

        let message = context.to_string();
        let stream = self.provider.chat(message).await?;
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
