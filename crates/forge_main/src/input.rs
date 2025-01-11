use anyhow::Result;
use inquire::Autocomplete;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

#[derive(Debug, EnumIter, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum UserInput {
    End,
    New,
    Message(String),
}

#[derive(Clone)]
struct CommandCompleter {
    commands: Vec<String>,
}

impl CommandCompleter {
    fn new() -> Self {
        Self { commands: UserInput::available_commands() }
    }
}

impl Autocomplete for CommandCompleter {
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        if input.starts_with('/') {
            Ok(self
                .commands
                .iter()
                .filter(|cmd| cmd.starts_with(input))
                .cloned()
                .collect())
        } else {
            Ok(vec![])
        }
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(highlighted_suggestion.or_else(|| {
            if input.starts_with('/') {
                self.commands
                    .iter()
                    .find(|cmd| cmd.starts_with(input))
                    .cloned()
            } else {
                None
            }
        }))
    }
}

impl UserInput {
    pub fn prompt() -> Result<Self> {
        loop {
            let text = inquire::Text::new("")
                .with_help_message(&format!(
                    "Available commands: {}",
                    Self::available_commands().join(", ")
                ))
                .with_autocomplete(CommandCompleter::new())
                .prompt()?;

            match Self::parse(&text)? {
                None => {
                    // Show help text and continue the loop
                    println!("\n{}\n", Self::get_help_text());
                    continue;
                }
                Some(input) => return Ok(input),
            }
        }
    }

    pub fn prompt_initial() -> Result<String> {
        Ok(inquire::Text::new("")
            .with_help_message("How can I help?")
            .prompt()?
            .to_string())
    }

    fn get_help_text() -> String {
        "Available Commands:
/help  - Display this help message
/new   - Start a new conversation
/end   - End the current conversation and exit

Other Usage:
- Type your message directly to chat with the AI
- Commands are case-sensitive and must start with '/'
- Use tab completion to quickly enter commands
- Arrow keys can be used to navigate command history"
            .to_string()
    }

    pub fn parse(input: &str) -> Result<Option<Self>> {
        let trimmed = input.trim();
        match trimmed {
            "/end" => Ok(Some(UserInput::End)),
            "/new" => Ok(Some(UserInput::New)),
            "/help" => Ok(None), // Return None to indicate help should be shown
            cmd if cmd.starts_with('/') => {
                let available_commands = Self::available_commands();
                Err(anyhow::anyhow!(
                    "Unknown command: '{}'. Available commands: {}",
                    cmd,
                    available_commands.join(", ")
                ))
            }
            text => Ok(Some(UserInput::Message(text.to_string()))),
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
