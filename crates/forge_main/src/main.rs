use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use forge::{Cli, UI};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize and run the UI
    let cli = Cli::parse();
    let api = Arc::new(forge_api::ForgeAPI::init(cli.restricted));
    let mut ui = UI::init(cli, api).await?;
    ui.run().await?;

    Ok(())
}
