use anyhow::Result;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

#[derive(Debug, EnumIter, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum UserInput {
    End,
    New,
    Message(String),
}

impl UserInput {
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        match trimmed {
            "/end" => Ok(UserInput::End),
            "/new" => Ok(UserInput::New),
            cmd if cmd.starts_with('/') => {
                let available_commands = Self::available_commands();
                Err(anyhow::anyhow!(
                    "Unknown command: '{}'. Available commands: {}",
                    cmd,
                    available_commands.join(", ")
                ))
            }
            text => Ok(UserInput::Message(text.to_string())),
        }
    }

    pub fn available_commands() -> Vec<String> {
        UserInput::iter()
            .filter_map(|cmd| match cmd {
                UserInput::Message(_) => None,
                cmd => Some(format!("/{}", cmd.as_ref())),
            })
            .collect()
    }
}
