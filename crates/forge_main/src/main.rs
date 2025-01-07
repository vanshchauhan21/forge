use clap::Parser;
use colored::*;
use forge_server::{ChatResponse, Result, API};
use tokio_stream::StreamExt;

#[derive(Parser)]
struct Cli {
    path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(path) = cli.path {
        let content = tokio::fs::read_to_string(path).await?;

        println!("\r{}", content);

        let mut stream = API::init().await?.run(content).await?;
        while let Some(message) = stream.next().await {
            match message {
                ChatResponse::Text(text) => {
                    print!("{}", text);
                }
                ChatResponse::ToolUseDetected(_) => {}
                ChatResponse::ToolCallStart(tool_call_full) => {
                    println!(
                        "{} {}",
                        "Tool use detected:".green(),
                        tool_call_full.name.as_str()
                    );
                }
                ChatResponse::ToolCallEnd(tool_result) => {
                    println!("{}", tool_result);
                }
                ChatResponse::ConversationStarted(conversation_id) => {
                    println!("Job {} started", conversation_id.as_uuid());
                }
                ChatResponse::ModifyContext(_) => {}
                ChatResponse::Complete => {
                    println!("Job completed");
                }
                ChatResponse::Error(_) => {}
            }
        }

        Ok(())
    } else {
        Ok(API::init().await?.launch().await?)
    }
}
