use std::path::Path;

use clap::Parser;
use forge_cli::command::Cli;
use forge_cli::{Engine, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with level from CLI
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level.unwrap_or_default())
        .init();

    let engine = Engine::new(cli.key, Path::new(".").to_path_buf());
    engine.launch().await?;

    Ok(())
}
