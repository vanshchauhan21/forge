use std::error::Error as StdError;
use std::path::PathBuf;

use async_trait::async_trait;
use forge_domain::{Command, Usage, UserInput};
use inquire::Autocomplete;
use tokio::fs;

use crate::console::CONSOLE;
use crate::StatusDisplay;

/// Provides command autocompletion functionality for the input prompt.
///
/// This struct maintains a list of available commands and implements
/// the Autocomplete trait to provide suggestions as the user types.
#[derive(Clone)]
struct CommandCompleter {
    commands: Vec<String>,
}

/// Console implementation for handling user input via command line.
#[derive(Debug, Default)]
pub struct Console;

impl CommandCompleter {
    /// Creates a new command completer with the list of available commands
    fn new() -> Self {
        Self { commands: Command::available_commands() }
    }
}

impl Autocomplete for CommandCompleter {
    /// Returns a list of command suggestions that match the current input.
    /// Only provides suggestions for inputs starting with '/'.
    fn get_suggestions(
        &mut self,
        input: &str,
    ) -> std::result::Result<Vec<String>, Box<dyn StdError + Send + Sync>> {
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

    /// Returns the best matching command completion for the current input.
    /// Only provides completions for inputs starting with '/'.
    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> std::result::Result<Option<String>, Box<dyn StdError + Send + Sync>> {
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

#[async_trait]
impl UserInput for Console {
    async fn upload<P: Into<PathBuf> + Send>(&self, path: P) -> anyhow::Result<Command> {
        let path = path.into();
        let content = fs::read_to_string(&path).await?.trim().to_string();

        CONSOLE.writeln(content.clone())?;
        Ok(Command::Message(content))
    }

    async fn prompt(
        &self,
        help_text: Option<&str>,
        initial_text: Option<&str>,
    ) -> anyhow::Result<Command> {
        loop {
            CONSOLE.writeln("")?;
            let help = help_text.map(|a| a.to_string()).unwrap_or(format!(
                "How can I help? Available commands: {}",
                Command::available_commands().join(", ")
            ));

            let mut text = inquire::Text::new("")
                .with_help_message(&help)
                .with_autocomplete(CommandCompleter::new());

            if let Some(initial_text) = initial_text {
                text = text.with_initial_value(initial_text);
            }

            let text = text.prompt()?;
            match Command::parse(&text) {
                Ok(input) => return Ok(input),
                Err(e) => {
                    CONSOLE
                        .writeln(StatusDisplay::failed(e.to_string(), Usage::default()).format())?;
                }
            }
        }
    }
}
