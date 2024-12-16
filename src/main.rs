mod core;
mod error;

use clap::Parser;
use core::Engine;
use error::Result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// API Key to be used
    key: String,

    /// Model to be used
    model: String,

    /// Base URL to be used
    base_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{:?}", cli);

    // Initialize chat engine
    let engine = Engine::new(cli.key);

    // Testing if the connection is successful
    engine.test().await?;

    Ok(())
}
