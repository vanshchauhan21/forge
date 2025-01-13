use anyhow::Result;
use async_trait::async_trait;
use forge_domain::UserInteractionPort;

pub struct ConsoleUserInteraction {
    // Implementation details will be added later
}

impl Default for ConsoleUserInteraction {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleUserInteraction {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UserInteractionPort for ConsoleUserInteraction {
    async fn get_input(&self, _prompt: &str) -> Result<String> {
        // Implementation will be added later
        todo!()
    }

    async fn get_confirmation(&self, _prompt: &str) -> Result<bool> {
        // Implementation will be added later
        todo!()
    }

    async fn get_selection(&self, _prompt: &str, _options: &[String]) -> Result<String> {
        // Implementation will be added later
        todo!()
    }
}
