use std::path::PathBuf;

use async_trait::async_trait;

use crate::error::{Error, Result};

/// Represents user input types in the chat application.
///
/// This enum encapsulates all forms of input including:
/// - System commands (starting with '/')
/// - Regular chat messages
/// - File content
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// End the current session and exit the application.
    /// This can be triggered with the '/end' command.
    End,
    /// Start a new conversation while preserving history.
    /// This can be triggered with the '/new' command.
    New,
    /// Reload the conversation with the original prompt.
    /// This can be triggered with the '/reload' command.
    Reload,
    /// A regular text message from the user to be processed by the chat system.
    /// Any input that doesn't start with '/' is treated as a message.
    Message(String),
    /// Display system environment information.
    /// This can be triggered with the '/info' command.
    Info,
}

impl Command {
    /// Returns a list of all available command strings.
    ///
    /// These commands are used for:
    /// - Command validation
    /// - Autocompletion
    /// - Help display
    pub fn available_commands() -> Vec<String> {
        vec![
            "/end".to_string(),
            "/new".to_string(),
            "/reload".to_string(),
            "/info".to_string(),
        ]
    }

    /// Parses a string input into an Input.
    ///
    /// This function:
    /// - Trims whitespace from the input
    /// - Recognizes and validates commands (starting with '/')
    /// - Converts regular text into messages
    ///
    /// # Returns
    /// - `Ok(Input)` - Successfully parsed input
    /// - `Err` - Input was an invalid command
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        match trimmed {
            "/end" => Ok(Command::End),
            "/new" => Ok(Command::New),
            "/reload" => Ok(Command::Reload),
            "/info" => Ok(Command::Info),
            cmd if cmd.starts_with('/') => Err(Error::InputCommand(cmd.to_string())),
            text => Ok(Command::Message(text.to_string())),
        }
    }
}

/// A trait for handling user input in the application.
///
/// This trait defines the core functionality needed for processing
/// user input, whether it comes from a command line interface,
/// GUI, or file system.
#[async_trait]
pub trait UserInput {
    /// Read content from a file and convert it to the input type.
    ///
    /// # Arguments
    /// * `path` - The path to the file to read
    ///
    /// # Returns
    /// * `Ok(Input)` - Successfully read and parsed file content
    /// * `Err` - Failed to read or parse file
    async fn upload<P: Into<PathBuf> + Send>(&self, path: P) -> anyhow::Result<Command>;

    /// Prompts for user input with optional help text and initial value.
    ///
    /// # Arguments
    /// * `help_text` - Optional help text to display with the prompt
    /// * `initial_text` - Optional initial text to populate the input with
    ///
    /// # Returns
    /// * `Ok(Input)` - Successfully processed input
    /// * `Err` - An error occurred during input processing
    async fn prompt(
        &self,
        help_text: Option<&str>,
        initial_text: Option<&str>,
    ) -> anyhow::Result<Command>;
}
