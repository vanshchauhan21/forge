mod core;
mod error;
mod ui;

use core::Engine;
use error::Result;
use ui::ChatUI;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize chat engine
    let engine = Engine::new(
        "You are an AI assistant with expertise in programming, software development, and technology. \
        You excel at providing clear, accurate, and well-structured responses. When discussing code, you use proper \
        formatting and explain key concepts thoroughly. You are direct and professional, focusing on delivering \
        high-quality technical assistance while maintaining a helpful demeanor.".to_string()     
    );

    // Testing if the connection is successful
    engine.test().await?;

    // Create chat UI and channels
    let chat_ui = ChatUI::new(engine);

    // Start the chat interface
    chat_ui.run().await?;

    Ok(())
}
