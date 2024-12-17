use clap::Parser;
use error::Result;
use forge_cli::{
    command::{Cli, Mode},
    completion::Completion,
    error,
};
use forge_provider::Provider;
use futures::StreamExt;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with level from CLI
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level.unwrap_or_default())
        .init();

    // Initialize chat engine
    let provider = Provider::open_router(cli.key, cli.model.clone(), cli.base_url.clone());

    // Testing if the connection is successful
    provider.test().await?;

    let mut mode = Mode::default();

    loop {
        let prompt = inquire::Text::new(format!("{} ❯", mode).as_str())
            .with_autocomplete(Completion::new(Mode::variants()))
            .prompt()?;

        if prompt.starts_with("/") {
            if let Ok(input) = prompt.trim_start_matches("/").parse::<Mode>() {
                if matches!(input, Mode::Quit) {
                    break;
                }

                mode = input;
            }

            continue;
        }

        let mut output = provider.prompt(prompt).await?;

        while let Some(text) = output.next().await {
            print!("{}", text?);
        }

        println!();

        std::io::stdout().flush().unwrap();
    }

    Ok(())
}
