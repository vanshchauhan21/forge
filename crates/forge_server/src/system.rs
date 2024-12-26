use forge_provider::{AnyMessage, Message};
use handlebars::Handlebars;
use serde::Serialize;

use crate::Result;

#[derive(Serialize)]
pub struct SystemPrompt {
    operating_system: String,
    current_working_dir: String,
}

impl SystemPrompt {
    pub fn build() -> Result<Self> {
        Ok(Self {
            operating_system: std::env::consts::OS.to_string(),
            current_working_dir: std::env::current_dir()?.to_string_lossy().to_string(),
        })
    }
}

impl From<SystemPrompt> for AnyMessage {
    fn from(value: SystemPrompt) -> Self {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);

        let message = hb
            .render_template(include_str!("./prompts/system.md"), &value)
            .expect("Failed to render system prompt");

        AnyMessage::System(Message::system(message))
    }
}
