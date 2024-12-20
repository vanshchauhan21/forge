use crate::model::State;
use crate::{error::Result, model::Event};
use forge_provider::model::{Message, Request};
use forge_provider::{Provider, Stream};
use forge_tool::{Prompt, Router};
use std::sync::{Arc, Mutex};

pub struct CodeForge {
    state: Arc<Mutex<State>>,
    tool_engine: Router,
    provider: Provider,
}

impl CodeForge {
    pub fn new(key: String) -> Self {
        // Add initial set of tools

        CodeForge {
            state: Arc::new(Mutex::new(State::default())),
            // TODO: add fs and think
            tool_engine: Router::default(),

            // TODO: make the provider configurable
            provider: Provider::open_router(key, None, None),
        }
    }

    pub async fn chat(&self, prompt: Prompt) -> Result<Stream<Event>> {
        // - Create Request, update context
        //   -  Add System Message [DONE]
        //   -  Add Add all tools [DONE]
        //   -  Add User Message [DONE]
        //   -  Add Context Files [DONE]
        // - Send message to LLM and await response #001 [DONE]
        // - On Response, dispatch event
        // - Check response has tool_use
        // - Execute tool
        // - Dispatch Event
        // - Add tool response to context
        // - Goto #001

        // let (tx, rx) = tokio::sync::mpsc::channel(1);

        // TODO: add message to history
        let context = Request::default()
            .add_message(Message::system(include_str!("./prompt.md").to_string()))
            .extend_tools(self.tool_engine.list())
            .add_message(Message::user(prompt.message))
            .extend_messages(
                prompt
                    .files
                    .into_iter()
                    .map(|f| Message::user(format!("{}\n{}", f.path, f.content)))
                    .collect(),
            );

        // TODO: Streaming is making the design complicated
        let response = self.provider.chat(context).await?;

        // TODO: need to handle errors more concisely
        todo!()
    }

    pub fn model(self, model: String) -> Self {
        // TODO: update the provider to use the passed model
        self
    }

    pub async fn models(&self) -> Result<Vec<String>> {
        Ok(self.provider.models().await?)
    }

    /// Resets the state of the forge without changing the model
    pub fn reset(self) -> Self {
        todo!()
    }
}
