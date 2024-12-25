use crate::completion::Completion;
use crate::conversation::Conversation;

// Shared state between each request to the server
pub struct App {
    pub completion: Completion,
    pub conversation: Conversation,
}

impl App {
    pub fn new(api_key: String, working_dir: impl Into<String>) -> Self {
        Self {
            completion: Completion::new(working_dir),
            conversation: Conversation::new(api_key),
        }
    }
}
