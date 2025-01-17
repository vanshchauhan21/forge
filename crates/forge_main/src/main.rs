use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use forge_app::Routes;
use forge_domain::{ChatRequest, ChatResponse, Command, ModelId, UserInput};
use forge_main::{display_info, Console, StatusDisplay, CONSOLE};
use tokio_stream::StreamExt;

fn context_reset_message(_: &Command) -> String {
    "All context was cleared, and we're starting fresh. Please re-add files and details so we can get started.".to_string()
        .yellow()
        .bold()
        .to_string()
}

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

    // Display the banner in dimmed colors
    forge_main::banner::display()?;

    let mut current_conversation_id = None;
    let mut current_title = None;
    let mut current_content = None;

    let api = Routes::init()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize API: {}", e))?;

    // Create a Console instance
    let console = Console;

    // Get initial input from file or prompt
    let mut input = match &cli.exec {
        Some(ref path) => console.upload(path).await?,
        None => console.prompt(None, None).await?,
    };
    let model = ModelId::from_env(&api.environment().await?);
    loop {
        match input {
            Command::End => break,
            Command::New => {
                CONSOLE.writeln(context_reset_message(&input))?;
                current_conversation_id = None;
                current_title = None;
                input = console.prompt(None, None).await?;
                continue;
            }
            Command::Reload => {
                CONSOLE.writeln(context_reset_message(&input))?;
                current_conversation_id = None;
                current_title = None;
                input = match cli.exec {
                    Some(ref path) => console.upload(path).await?,
                    None => console.prompt(None, current_content.as_deref()).await?,
                };
                continue;
            }
            Command::Info => {
                display_info(&api.environment().await?)?;
                input = console.prompt(current_title.as_deref(), None).await?;
                continue;
            }
            Command::Message(ref content) => {
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

                                        // Convert to JSON and apply dimmed style
                                        let json =
                                            serde_json::to_string_pretty(&tool_call_full.arguments)
                                                .unwrap_or_else(|_| {
                                                    "Failed to serialize arguments".to_string()
                                                });

                                        CONSOLE.writeln(format!("{}", json.dimmed()))?;
                                    }
                                    ChatResponse::ToolCallEnd(tool_result) => {
                                        let tool_name = tool_result.name.as_str();
                                        // Always show result content for errors, or in verbose mode
                                        if tool_result.is_error || cli.verbose {
                                            CONSOLE.writeln(format!(
                                                "{}",
                                                tool_result.to_string().dimmed()
                                            ))?;
                                        }
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

                input = console.prompt(current_title.as_deref(), None).await?;
            }
        }
    }

    Ok(())
}
