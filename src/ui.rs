use crate::error::Result;

use crate::core::ChatEngine;

pub struct ChatUI {
    engine: ChatEngine,
}

impl ChatUI {
    pub fn new(engine: ChatEngine) -> Self {
        ChatUI { engine }
    }

    pub async fn run(&self) -> Result<()> {
        todo!()
    }
}
