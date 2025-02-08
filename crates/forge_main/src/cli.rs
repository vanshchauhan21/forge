use std::fs;
use std::path::PathBuf;

use clap::Parser;

/// Command line arguments for the application
#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Optional file path to execute commands from
    #[arg(long, short = 'c')]
    pub command: Option<String>,
    /// Enable verbose output, showing additional tool information
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
    /// Path to custom instructions
    #[arg(long, short = 'i',value_parser = validate_path)]
    pub custom_instructions: Option<PathBuf>,
    /// Path to the system prompt file
    #[arg(
        long,
        short = 's',
        value_parser = validate_path
    )]
    pub system_prompt: Option<PathBuf>,
}

fn validate_path(path: &str) -> Result<PathBuf, String> {
    let path_buf = PathBuf::from(path);

    // check if the path exists
    if !path_buf.exists() {
        return Err(format!("Path does not exist: '{}'", path_buf.display()));
    }

    // Check if it's a file
    if !path_buf.is_file() {
        return Err(format!("Path is not a file: '{}'", path_buf.display()));
    }

    // Check if readable by attempting to read metadata
    if fs::metadata(&path_buf).is_err() {
        return Err(format!(
            "Unable to read file from path '{}'",
            path_buf.display()
        ));
    }
    Ok(path_buf)
}
