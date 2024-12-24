use crate::completion::Completion;
use crate::engine::Engine;

// Shared state between each request to the server
pub struct App {
    pub completion: Completion,
    pub engine: Engine,
}

impl App {
    pub fn new(working_dir: impl Into<String>) -> Self {
        let engine = Engine::default();
        Self { completion: Completion::new(working_dir), engine }
    }
}
