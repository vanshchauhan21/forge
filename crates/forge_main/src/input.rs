use anyhow::Result;
use inquire::Autocomplete;
use strum_macros::AsRefStr;

#[derive(Debug)]
pub enum UserInput {
    End,
    New,
    Message(String),
}

#[derive(Debug, AsRefStr)]
#[strum(serialize_all = "lowercase")]
enum InputKind {
    Help,
    User(UserInput),
}

#[derive(Clone)]
struct CommandCompleter {
    commands: Vec<String>,
}

impl CommandCompleter {
    fn new() -> Self {
        Self { commands: InputKind::available_commands() }
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

impl InputKind {
    fn available_commands() -> Vec<String> {
        vec!["/end".to_string(), "/new".to_string(), "/help".to_string()]
    }

    fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        match trimmed {
            "/end" => Ok(InputKind::User(UserInput::End)),
            "/new" => Ok(InputKind::User(UserInput::New)),
            "/help" => Ok(InputKind::Help),
            cmd if cmd.starts_with('/') => {
                let available_commands = Self::available_commands();
                Err(anyhow::anyhow!(
                    "Unknown command: '{}'. Available commands: {}",
                    cmd,
                    available_commands.join(", ")
                ))
            }
            text => Ok(InputKind::User(UserInput::Message(text.to_string()))),
        }
    }

    fn get_help_text() -> String {
        "Available Commands:
/new   - Start a new conversation
/end   - End the current conversation and exit

Other Usage:
- Type your message directly to chat with the AI
- Commands are case-sensitive and must start with '/'
- Use tab completion to quickly enter commands
- Arrow keys can be used to navigate command history"
            .to_string()
    }
}

impl UserInput {
    pub fn prompt() -> Result<Self> {
        loop {
            let text = inquire::Text::new("")
                .with_help_message(&format!(
                    "Available commands: {}",
                    InputKind::available_commands().join(", ")
                ))
                .with_autocomplete(CommandCompleter::new())
                .prompt()?;

            match InputKind::parse(&text)? {
                InputKind::Help => {
                    println!("\n{}\n", InputKind::get_help_text());
                    continue;
                }
                InputKind::User(input) => return Ok(input),
            }
        }
    }

    pub fn prompt_initial() -> Result<String> {
        loop {
            let text = inquire::Text::new("")
                .with_help_message(&format!(
                    "How can I help? Available commands: {}",
                    InputKind::available_commands().join(", ")
                ))
                .with_autocomplete(CommandCompleter::new())
                .prompt()?;

            match InputKind::parse(&text)? {
                InputKind::Help => {
                    println!("\n{}\n", InputKind::get_help_text());
                    continue;
                }
                InputKind::User(UserInput::Message(msg)) => return Ok(msg),
                InputKind::User(_) => continue, // Ignore other commands in initial prompt
            }
        }
    }
}
