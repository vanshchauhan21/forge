use clap::Parser;
use colorize::AnsiColor;
use error::Result;
use forge_cli::{
    command::{Cli, Mode},
    completion::Completion,
    error,
};
use forge_engine::{CodeForge, Event};
use futures::StreamExt;
use spinners::{Spinner, Spinners};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with level from CLI
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level.unwrap_or_default())
        .init();

    let mut agent = CodeForge::new(cli.key.clone());
    let mut mode = Mode::default();
    let mut end = false;
    while !end {
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
                        let models = agent.models().await?;
                        let input = inquire::Select::new("Choose a model", models).prompt()?;
                        agent = agent.model(input)
                    }
                }
            }

            continue;
        }

        let mut spinner = Spinner::new(Spinners::Dots9, "Thinking...".into());
        let mut output = agent.prompt(prompt).await?;
        let buffer = String::new();
        while let Some(event) = output.next().await {
            match event {
                Event::Inquire(_) => todo!(),
                Event::Text(text) => {
                    println!("{}", text);
                }
                Event::End => {
                    end = true;
                    break;
                }
            }
        }
        spinner.stop_with_message("Here is what I thought...".into());

        println!("{}", buffer);
    }

    Ok(())
}
