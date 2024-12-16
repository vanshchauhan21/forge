mod core;
mod error;

use clap::Parser;
use core::Engine;
use error::Result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// API Key to be used
    #[arg(short, long)]
    key: String,

    /// Model to be used
    #[arg(short, long)]
    model: String,

    /// Base URL to be used
    #[arg(short, long)]
    base_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize chat engine
    let engine = Engine::new(cli.key, cli.model, cli.base_url);

    // Testing if the connection is successful
    engine.test().await?;

    Ok(())
}
