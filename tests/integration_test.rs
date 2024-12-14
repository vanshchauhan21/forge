//! Integration tests for Code Forge Chat
//! 
//! These tests verify the connection to OpenRouter and basic functionality
//! of the chat engine. To run these tests, make sure you have set the
//! OPENROUTER_API_KEY environment variable.

use code_forge::ChatEngine;
use std::env;

/// Test the connection to OpenRouter and verify we can access Claude 3 Sonnet
#[tokio::test]
async fn test_openrouter_connection() {
    // Verify API key is set
    if env::var("OPENROUTER_API_KEY").is_err() {
        panic!("❌ OPENROUTER_API_KEY environment variable is not set");
    }

    let chat_engine = ChatEngine::new(
        "You are a test assistant.".to_string(),
        "Test connection".to_string()
    );

    match chat_engine.test_connection().await {
        Ok(response) => {
            assert!(
                response.contains("Connected successfully"),
                "Expected connection test response, got: {}",
                response
            );
            println!("✅ Successfully connected to OpenRouter");
            println!("Response: {}", response);
        },
        Err(e) => {
            panic!("❌ Failed to connect to OpenRouter: {}\n\
                   \nTroubleshooting steps:\n\
                   1. Verify your API key is valid\n\
                   2. Check if you have access to 'anthropic/claude-3-sonnet-20240229'\n\
                   3. Visit https://openrouter.ai/docs#models for supported models\n\
                   4. Check your internet connection\n", 
                   e);
        }
    }
}

/// Helper function to run the test with proper environment setup
#[tokio::test]
async fn test_chat_engine_initialization() {
    let chat_engine = ChatEngine::new(
        "Test system prompt".to_string(),
        "Test user prompt".to_string()
    );

    assert_eq!(
        chat_engine.test_connection().await.is_ok(),
        true,
        "Chat engine should initialize and connect successfully"
    );
}
