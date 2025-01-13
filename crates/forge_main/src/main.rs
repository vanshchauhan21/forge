use anyhow::Result;
use clap::Parser;
use forge_domain::{ChatRequest, ChatResponse, ModelId};
use forge_main::{StatusDisplay, UserInput, CONSOLE};
use forge_server::API;
use tokio_stream::StreamExt;

/// Command line arguments for the application
#[derive(Parser)]
struct Cli {
    /// Optional file path to execute commands from
    exec: Option<String>,
    /// Enable verbose output, showing additional tool information
    #[arg(long, default_value_t = false)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut current_conversation_id = None;
    let mut current_title = None;
    let mut current_content = None;

    let api = API::init()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize API: {}", e))?;

    // Get initial input from file or prompt
    let mut input = match &cli.exec {
        Some(ref path) => UserInput::from_file(path).await?,
        None => UserInput::prompt(None, None)?,
    };
    let model = ModelId::from_env(api.env());
    loop {
        match input {
            UserInput::End => break,
            UserInput::New => {
                CONSOLE.writeln("Starting fresh conversation...")?;
                current_conversation_id = None;
                current_title = None;
                input = UserInput::prompt(None, None)?;
                continue;
            }
            UserInput::Reload => {
                CONSOLE.writeln("Reloading conversation with original prompt...")?;
                current_conversation_id = None;
                current_title = None;
                input = match cli.exec {
                    Some(ref path) => UserInput::from_file(path).await?,
                    None => UserInput::prompt(None, current_content.as_deref())?,
                };
                continue;
            }
            UserInput::Message(ref content) => {
                current_content = Some(content.clone());
                let chat = ChatRequest {
                    content: content.clone(),
                    model: model.clone(),
                    conversation_id: current_conversation_id,
                };

                match api.chat(chat).await {
                    Ok(mut stream) => {
                        while let Some(message) = stream.next().await {
                            match message {
                                Ok(message) => match message {
                                    ChatResponse::Text(text) => {
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
                                        CONSOLE.newline()?;
                                        CONSOLE
                                            .writeln(StatusDisplay::execute(tool_name).format())?;
                                    }
                                    ChatResponse::ToolCallEnd(tool_result) => {
                                        if cli.verbose {
                                            CONSOLE.writeln(tool_result.to_string())?;
                                        }
                                        let tool_name = tool_result.name.as_str();
                                        let status = if tool_result.is_error {
                                            StatusDisplay::failed(tool_name)
                                        } else {
                                            StatusDisplay::success(tool_name)
                                        };
                                        CONSOLE.write(status.format())?;
                                    }
                                    ChatResponse::ConversationStarted(conversation_id) => {
                                        current_conversation_id = Some(conversation_id);
                                    }
                                    ChatResponse::ModifyContext(_) => {}
                                    ChatResponse::Complete => {}
                                    ChatResponse::Error(err) => {
                                        CONSOLE.writeln(
                                            StatusDisplay::failed(err.to_string()).format(),
                                        )?;
                                    }
                                    ChatResponse::PartialTitle(_) => {}
                                    ChatResponse::CompleteTitle(title) => {
                                        current_title = Some(StatusDisplay::title(title).format());
                                    }
                                    ChatResponse::FinishReason(_) => {}
                                },
                                Err(err) => {
                                    CONSOLE
                                        .writeln(StatusDisplay::failed(err.to_string()).format())?;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        CONSOLE.writeln(
                            StatusDisplay::failed_with(
                                err.to_string(),
                                "Failed to establish chat stream",
                            )
                            .format(),
                        )?;
                    }
                }

                input = UserInput::prompt(current_title.as_deref(), None)?;
            }
        }
    }

    Ok(())
}
