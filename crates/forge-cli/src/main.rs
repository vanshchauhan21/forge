use clap::Parser;
use colorize::AnsiColor;
use error::Result;
use forge_cli::{
    command::{Cli, Mode},
    completion::Completion,
    error,
    runtime::Runtime,
};
use forge_engine::CodeForge;
use forge_provider::Provider;
use futures::StreamExt;
use spinners::{Spinner, Spinners};

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
        let prompt = inquire::Text::new(format!("{}❯", mode).bold().as_str())
            .with_autocomplete(Completion::new(Mode::variants()))
            .with_help_message("Ask the agent to do something")
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

        let mut spinner = Spinner::new(Spinners::Dots9, "Thinking...".into());
        let mut output = provider.prompt(prompt).await?;
        let mut buffer = String::new();
        while let Some(text) = output.next().await {
            buffer.push_str(text?.as_str());
        }
        spinner.stop_with_message("Here is what I thought...".into());

        println!("{}", buffer);
    }

    CodeForge::default().run(Runtime::default()).await?;

    Ok(())
}
