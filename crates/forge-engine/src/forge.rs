use crate::model::State;
use crate::{error::Result, model::Event};
use derive_setters::Setters;
use forge_provider::model::{Message, Request, User};
use forge_provider::{Provider, Stream};
use forge_tool::ToolTrait;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub struct CodeForge {
    state: Arc<Mutex<State>>,
    tools: Vec<Rc<dyn ToolTrait>>,
    provider: Provider,
}

#[derive(Setters, Clone)]
pub struct Prompt {
    message: String,
    files: Vec<File>,
}

#[derive(Setters, Clone)]
pub struct File {
    pub path: String,
    pub content: String,
}

impl Prompt {
    pub fn new(message: String) -> Self {
        Prompt {
            message,
            files: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file: File) {
        self.files.push(file);
    }
}

impl CodeForge {
    pub fn new(key: String) -> Self {
        // Add initial set of tools
        let tools = vec![
            Rc::new(forge_tool::FS) as Rc<dyn ToolTrait>,
            Rc::new(forge_tool::Think::default()) as Rc<dyn ToolTrait>,
        ];

        CodeForge {
            state: Arc::new(Mutex::new(State::default())),
            // TODO: add fs and think
            tools,

            // TODO: make the provider configurable
            provider: Provider::open_router(key, None, None),
        }
    }

    pub fn add_tool<T: ToolTrait + Sync + 'static>(&mut self, tool: T) {
        self.tools.push(Rc::new(tool));
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
            // .extend_tools(self.tools.clone())
            .add_message(Message::user(prompt.message))
            .extend_messages(
                prompt
                    .files
                    .into_iter()
                    .map(Message::<User>::from)
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
