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
        // Send welcome message
        let _ = tx.send("Welcome! Type a message to begin.".to_string()).await;
        
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
    // Initialize chat components
    let chat_engine = ChatEngine::new(
        "You are a helpful AI assistant. Provide clear, concise responses.".to_string(),
        "Start a conversation".to_string()
    );
    
    let mut chat_ui = ChatUI::new()?;

    // Create channels for user input
    let (input_tx, input_rx) = mpsc::channel(100);

    // Create response stream
    let response_stream = create_response_stream(chat_engine, input_rx).await;

    // Start the chat interface
    chat_ui.run(response_stream).await?;

    Ok(())
}
