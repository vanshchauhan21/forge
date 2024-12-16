mod chat_engine;
mod ui;

use std::error::Error;
use futures::stream::Stream;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use chat_engine::ChatEngine;
use ui::ChatUI;

async fn create_response_stream(
    mut chat_engine: ChatEngine,
    mut input_rx: mpsc::Receiver<String>,
) -> impl Stream<Item = String> + Unpin {
    let (tx, rx) = mpsc::channel(100);
    
    // Spawn a task to process inputs and generate responses
    tokio::spawn(async move {
        // Process incoming messages
        while let Some(input) = input_rx.recv().await {
            let mut response_stream = chat_engine.process_message(input).await;
            
            while let Some(response_part) = response_stream.next().await {
                let _ = tx.send(response_part).await;
            }
        }
    });

    ReceiverStream::new(rx)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize chat engine
    let chat_engine = ChatEngine::new(
        "You are Claude 3 Sonnet, an AI assistant with expertise in programming, software development, and technology. \
        You excel at providing clear, accurate, and well-structured responses. When discussing code, you use proper \
        formatting and explain key concepts thoroughly. You are direct and professional, focusing on delivering \
        high-quality technical assistance while maintaining a helpful demeanor.".to_string(),
        "Start a conversation".to_string()
    );

    // Test connection before creating chat window
    match chat_engine.test_connection().await {
        Ok(_) => {
            println!("🟢 Successfully connected to Claude 3 Sonnet! Starting chat interface...");
            
            // Create chat UI and channels
            let mut chat_ui = ChatUI::new()?;
            let (input_tx, input_rx) = mpsc::channel(100);

            // Create response stream
            let response_stream = create_response_stream(chat_engine, input_rx).await;

            // Start the chat interface
            chat_ui.run(response_stream, input_tx).await?;
            Ok(())
        }
        Err(e) => {
            eprintln!("🔴 Failed to connect to Claude 3 Sonnet: {}", e);
            eprintln!("Please check your OPENROUTER_API_KEY environment variable and internet connection.");
            Ok(())
        }
    }
}
