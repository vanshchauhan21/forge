use core::panic;
use std::io::Write;

use clap::Parser;
use colored::Colorize;
use forge_domain::{ChatRequest, ChatResponse, ModelId};
use forge_server::{Result, API};
use tokio_stream::StreamExt;

#[derive(Parser)]
struct Cli {
    exec: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut content = if let Some(path) = cli.exec {
        let cwd = std::env::current_dir()?;
        let full_path = cwd.join(path);
        tokio::fs::read_to_string(full_path)
            .await?
            .trim()
            .to_string()
    } else {
        inquire::Text::new("")
            .with_help_message("How can I help?")
            .prompt()
            .unwrap()
            .to_string()
    };

    println!("{}", content.trim());
    let mut current_conversation_id = None;
    let api = API::init().await?;
    loop {
        let model = ModelId::from_env(api.env());
        let chat = ChatRequest {
            content: content.clone(),
            model,
            conversation_id: current_conversation_id,
        };

        let mut stream = api.chat(chat).await?;
        while let Some(message) = stream.next().await {
            match message.unwrap() {
                ChatResponse::Text(text) => {
                    print!("{}", text);
                }
                ChatResponse::ToolCallDetected(_) => {}
                ChatResponse::ToolCallArgPart(arg) => {
                    print!("{}", arg);
                }
                ChatResponse::ToolCallStart(tool_call_full) => {
                    println!(
                        "\n{} {}",
                        "Tool use detected:".green(),
                        tool_call_full.name.as_str()
                    );
                }
                ChatResponse::ToolCallEnd(tool_result) => {
                    println!("{}", tool_result);
                }
                ChatResponse::ConversationStarted(conversation_id) => {
                    current_conversation_id = Some(conversation_id);
                }
                ChatResponse::ModifyContext(_) => {}
                ChatResponse::Complete => {}
                ChatResponse::Error(err) => {
                    panic!("{:?}", err);
                }
                ChatResponse::PartialTitle(_) => {}
                ChatResponse::CompleteTitle(title) => {
                    println!("title: {}", title);
                }
            }

            std::io::stdout().flush().unwrap();
        }

        println!();
        content = inquire::Text::new("")
            .with_help_message("type '/done' to end this conversation.")
            .prompt()
            .unwrap();
        if content.trim() == "/done" {
            break;
        }
    }

    Ok(())
}
