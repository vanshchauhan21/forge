use std::io::Write;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use forge_domain::{ChatRequest, ChatResponse, ModelId};
use forge_server::API;
use tokio_stream::StreamExt;

mod input;
use input::UserInput;

#[derive(Parser)]
struct Cli {
    exec: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let initial_content = if let Some(path) = cli.exec {
        let cwd = std::env::current_dir()?;
        let full_path = cwd.join(path);
        tokio::fs::read_to_string(&full_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", full_path.display(), e))?
            .trim()
            .to_string()
    } else {
        UserInput::prompt_initial()?
    };

    println!("{}", initial_content.trim());
    let mut current_conversation_id = None;
    let api = API::init()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize API: {}", e))?;

    let mut content = initial_content;
    loop {
        let model = ModelId::from_env(api.env());
        let chat = ChatRequest {
            content: content.clone(),
            model,
            conversation_id: current_conversation_id,
        };

        let mut stream = api
            .chat(chat)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start chat stream: {}", e))?;

        while let Some(message) = stream.next().await {
            let message = message.map_err(|e| anyhow::anyhow!("Stream error: {}", e))?;
            match message {
                ChatResponse::Text(text) => {
                    print!("{}", text);
                }
                ChatResponse::ToolCallDetected(_) => {}
                ChatResponse::ToolCallArgPart(arg) => {
                    print!("{}", arg);
                }
                ChatResponse::ToolCallStart(tool_call_full) => {
                    println!(
                        "\n{} {} {} {}",
                        "▶".white(),
                        "TOOL USE DETECTED:".bold().white(),
                        tool_call_full.name.as_str(),
                        "◀".white()
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
                    return Err(anyhow::anyhow!("Chat error: {:?}", err));
                }
                ChatResponse::PartialTitle(_) => {}
                ChatResponse::CompleteTitle(title) => {
                    println!("{}", forge_main::format_title(&title));
                }
                ChatResponse::FinishReason(_) => {
                    println!();
                }
            }

            std::io::stdout()
                .flush()
                .map_err(|e| anyhow::anyhow!("Failed to flush stdout: {}", e))?;
        }

        println!();
        match UserInput::prompt()? {
            UserInput::End => break,
            UserInput::New => {
                println!("Starting fresh conversation...");
                current_conversation_id = None;
                content = UserInput::prompt_initial()?;
            }
            UserInput::Message(msg) => {
                content = msg;
            }
        }
    }

    Ok(())
}
