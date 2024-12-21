use std::path::Path;

use clap::Parser;
use error::Result;
use forge_cli::command::Cli;
use forge_cli::{error, Engine};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with level from CLI
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level.unwrap_or_default())
        .init();

    let engine = Engine::new(cli.key, Path::new(".").to_path_buf());
    engine.launch().await?;

    // let mut mode = Command::default();
    // loop {
    //     // TODO: we shouldn't get the latest files from fs on each loop, should
    // occur     // only when user is searching for files.

    //     let mut suggestions = ls_files(std::path::Path::new("."))
    //         .map(|v| v.into_iter().map(|a| format!("@{}",
    // a)).collect::<Vec<_>>())         .unwrap_or_default();
    //     suggestions.extend(Command::variants());

    //     let prompt = inquire::Text::new(format!("{}❯", mode).bold().as_str())
    //         .with_autocomplete(Completion::new(suggestions))
    //         .prompt()?;

    //     let mut spinner = Spinner::new(spinners::Spinners::Dots);

    //     let prompt = Prompt::parse(prompt).map_err(|e| e.to_string())?;

    //     let buffer = String::new();
    //     while let Some(event) = stream.next().await {
    //         spinner.stop();
    //         match event {
    //             Event::Ask(_) => todo!(),
    //             Event::Say(text) => {
    //                 print!("{}", text);
    //             }
    //             Event::Err(_) => todo!(),
    //         }
    //     }

    //     println!("{}", buffer);
    // }

    Ok(())
}

// struct Spinner {
//     spinner: spinners::Spinner,
//     is_done: bool,
// }

// impl Spinner {
//     pub fn new(dot: spinners::Spinners) -> Self {
//         let spinner = spinners::Spinner::new(dot, "".into());
//         Self { spinner, is_done: false }
//     }

//     pub fn stop(&mut self) {
//         if !self.is_done {
//             self.spinner
//                 .stop_with_message("Here is what I thought...".into());

//             self.is_done = true
//         }
//     }
// }

// fn ls_files(path: &std::path::Path) -> std::io::Result<Vec<String>> {
//     let mut paths = Vec::new();
//     let walker = WalkBuilder::new(path)
//         .hidden(true) // Skip hidden files
//         .git_global(true) // Use global gitignore
//         .git_ignore(true) // Use local .gitignore
//         .ignore(true) // Use .ignore files
//         .build();

//     for entry in walker.flatten() {
//         if entry.file_type().is_some_and(|ft| ft.is_file()) {
//             paths.push(entry.path().display().to_string());
//         }
//     }

//     Ok(paths)
// }
