use clap::Parser;
use error::Result;
use forge_cli::{
    command::{Cli, Mode},
    completion::Completion,
    error,
};
use forge_provider::Provider;
use futures::StreamExt;
use spinners::{Spinner, Spinners};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with level from CLI
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level.unwrap_or_default())
        .init();

    // Initialize chat engine
    let mut provider =
        Provider::open_router(cli.key.clone(), cli.model.clone(), cli.base_url.clone());

    // Testing if the connection is successful
    provider.test().await?;

    let mut mode = Mode::default();

    loop {
        let prompt = inquire::Text::new(format!("{}", mode).as_str())
            .with_autocomplete(Completion::new(Mode::variants()))
            .prompt()?;

        if prompt.starts_with("/") {
            if let Ok(prompt) = prompt.trim_start_matches("/").parse::<Mode>() {
                mode = prompt;
                match mode {
                    Mode::Ask => {}
                    Mode::Edit => {}
                    Mode::Quit => {
                        break;
                    }
                    Mode::Model => {
                        let models = provider.models().await?;
                        let input = inquire::Select::new("Choose a model", models).prompt()?;
                        provider = Provider::open_router(
                            cli.key.clone(),
                            Some(input),
                            cli.base_url.clone(),
                        )
                    }
                }
            }

            continue;
        }

        let mut sp = Spinner::new(Spinners::Dots9, "".into());
        let mut stop = true;
        let mut output = provider.prompt(prompt).await?;

        while let Some(text) = output.next().await {
            if stop {
                sp.stop_with_symbol("");
                stop = false

            }

            print!("{}", text?);
        }

        println!();

        std::io::stdout().flush().unwrap();
    }

    Ok(())
}
