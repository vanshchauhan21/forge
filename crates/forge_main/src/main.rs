use anyhow::Result;
use chrono::Local;
use clap::Parser;
use forge_domain::{ChatRequest, ChatResponse, ModelId};
use forge_main::{StatusDisplay, StatusKind, UserInput, CONSOLE};
use forge_server::API;
use tokio_stream::StreamExt;

#[derive(Parser)]
struct Cli {
    exec: Option<String>,
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

fn get_timestamp() -> String {
    Local::now().format("%H:%M:%S%.3f").to_string()
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

    CONSOLE.writeln(initial_content.trim())?;
    let mut current_conversation_id = None;
    let api = API::init()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize API: {}", e))?;

    let mut content = initial_content;
    let mut current_tool: Option<String> = None;

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
                    if current_tool.is_some() {
                        CONSOLE.writeln("")?;
                        current_tool = None;
                    }
                    CONSOLE.write(&text)?;
                }
                ChatResponse::ToolCallDetected(_) => {}
                ChatResponse::ToolCallArgPart(arg) => {
                    if cli.verbose {
                        CONSOLE.write(&arg)?;
                    }
                }
                ChatResponse::ToolCallStart(tool_call_full) => {
                    let tool_name = tool_call_full.name.as_str();
                    let status = StatusDisplay {
                        kind: StatusKind::Execute,
                        message: tool_name,
                        timestamp: Some(get_timestamp()),
                        error_details: None,
                    };
                    CONSOLE.writeln(status.format())?;
                    current_tool = Some(tool_name.to_string());
                }
                ChatResponse::ToolCallEnd(tool_result) => {
                    if cli.verbose {
                        CONSOLE.writeln(tool_result.to_string())?;
                    } else if let Some(tool_name) = &current_tool {
                        let status = if tool_result.is_error {
                            StatusDisplay {
                                kind: StatusKind::Failed,
                                message: tool_name,
                                timestamp: Some(get_timestamp()),
                                error_details: Some("error"),
                            }
                        } else {
                            StatusDisplay {
                                kind: StatusKind::Success,
                                message: tool_name,
                                timestamp: Some(get_timestamp()),
                                error_details: None,
                            }
                        };
                        CONSOLE.write(status.format())?;
                    }
                }
                ChatResponse::ConversationStarted(conversation_id) => {
                    current_conversation_id = Some(conversation_id);
                }
                ChatResponse::ModifyContext(_) => {}
                ChatResponse::Complete => {}
                ChatResponse::Error(err) => {
                    if current_tool.is_some() {
                        CONSOLE.writeln("")?;
                    }
                    return Err(anyhow::anyhow!("Chat error: {:?}", err));
                }
                ChatResponse::PartialTitle(_) => {}
                ChatResponse::CompleteTitle(title) => {
                    if current_tool.is_some() {
                        CONSOLE.writeln("")?;
                        current_tool = None;
                    }
                    let status = StatusDisplay {
                        kind: StatusKind::Title,
                        message: &title,
                        timestamp: Some(get_timestamp()),
                        error_details: None,
                    };
                    CONSOLE.writeln(status.format())?;
                }
                ChatResponse::FinishReason(_) => {
                    if current_tool.is_some() {
                        CONSOLE.writeln("")?;
                        current_tool = None;
                    }
                }
            }
        }

        match UserInput::prompt()? {
            UserInput::End => break,
            UserInput::New => {
                CONSOLE.writeln("Starting fresh conversation...")?;
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
