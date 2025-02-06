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
    #[arg(long, short = 'i')]
    pub custom_instructions: Option<PathBuf>,
}
