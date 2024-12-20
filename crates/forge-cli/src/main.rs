use clap::Parser;
use colorize::AnsiColor;
use error::Result;
use forge_cli::command::{Cli, Command};
use forge_cli::completion::Completion;
use forge_cli::error;
use forge_engine::model::Event;
use forge_engine::CodeForge;
use forge_tool::Prompt;
use futures::StreamExt;
use ignore::WalkBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with level from CLI
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level.unwrap_or_default())
        .init();

    let mut agent = CodeForge::new(cli.key.clone());
    let mut mode = Command::default();

    loop {
        // TODO: we shouldn't get the latest files from fs on each loop, should occur
        // only when user is searching for files.

        let mut suggestions = ls_files(std::path::Path::new("."))
            .map(|v| v.into_iter().map(|a| format!("@{}", a)).collect::<Vec<_>>())
            .unwrap_or_default();
        suggestions.extend(Command::variants());

        let prompt = inquire::Text::new(format!("{}❯", mode).bold().as_str())
            .with_autocomplete(Completion::new(suggestions))
            .prompt()?;

        if prompt.starts_with("/") {
            if let Ok(prompt) = prompt.trim_start_matches("/").parse::<Command>() {
                mode = prompt;
                match mode {
                    Command::Ask => {}
                    Command::Edit => {}
                    Command::Quit => {
                        break;
                    }
                    Command::Model => {
                        let models = agent.models().await?;
                        let input = inquire::Select::new("Choose a model", models).prompt()?;
                        agent = agent.model(input)
                    }
                }
            }

            continue;
        }

        let mut spinner = Spinner::new(spinners::Spinners::Dots);

        let prompt = Prompt::parse(prompt).await.map_err(|e| e.to_string())?;
        let mut stream = agent.chat(prompt).await?;

        let buffer = String::new();
        while let Some(event) = stream.next().await {
            spinner.stop();
            match event {
                Event::Ask(_) => todo!(),
                Event::Say(text) => {
                    print!("{}", text);
                }
                Event::Err(_) => todo!(),
            }
        }

        println!("{}", buffer);
    }

    Ok(())
}

struct Spinner {
    spinner: spinners::Spinner,
    message: String,
    is_done: bool,
}

impl Spinner {
    pub fn new(dot: spinners::Spinners) -> Self {
        let spinner = spinners::Spinner::new(dot, "".into());
        Self { spinner, message: "".into(), is_done: false }
    }

    pub fn stop(&mut self) {
        if !self.is_done {
            self.spinner
                .stop_with_message("Here is what I thought...".into());

            self.is_done = true
        }
    }
}

fn ls_files(path: &std::path::Path) -> std::io::Result<Vec<String>> {
    let mut paths = Vec::new();
    let walker = WalkBuilder::new(path)
        .hidden(true) // Skip hidden files
        .git_global(true) // Use global gitignore
        .git_ignore(true) // Use local .gitignore
        .ignore(true) // Use .ignore files
        .build();

    for result in walker {
        if let Ok(entry) = result {
            if entry.file_type().is_some_and(|ft| ft.is_file()) {
                paths.push(entry.path().display().to_string());
            }
        }
    }

    Ok(paths)
}
