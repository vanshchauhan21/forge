use crate::error::Result;

use crate::core::Engine;

pub struct ChatUI {
    engine: Engine,
}

impl ChatUI {
    pub fn new(engine: Engine) -> Self {
        ChatUI { engine }
    }

    pub async fn run(&self) -> Result<()> {
        todo!()
    }
}
