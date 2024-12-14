mod chat_engine;
mod chat_ui;

use std::error::Error;
use futures::stream::Stream;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use chat_engine::ChatEngine;
use chat_ui::ChatUI;

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
async fn main() -> Result<(), Box<dyn Error>> {
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
            let mut response_stream = create_response_stream(chat_engine, input_rx).await;

            // Create welcome message
            let (welcome_tx, welcome_rx) = mpsc::channel(1);
            welcome_tx.send(
                "Welcome to Code Forge Chat, powered by Claude 3 Sonnet.\n\
                I'm your AI programming assistant, specializing in software development and technical topics.\n\
                Feel free to ask questions about programming, architecture, best practices, or any tech-related topics.\n\
                Type your message and press Enter to send. Press Ctrl+C or Esc to exit.\n".to_string()
            ).await?;
            let welcome_stream = ReceiverStream::new(welcome_rx);

            // Combine welcome message with response stream
            let combined_stream = welcome_stream.chain(response_stream);

            // Start the chat interface
            chat_ui.run(combined_stream).await?;
            Ok(())
        }
        Err(e) => {
            eprintln!("🔴 Failed to connect to Claude 3 Sonnet: {}", e);
            eprintln!("Please check your OPENROUTER_API_KEY environment variable and internet connection.");
            Err(e.into())
        }
    }
}
