//! Integration tests for Code Forge Chat
//! 
//! These tests verify the connection to OpenRouter and basic functionality
//! of the chat engine. To run these tests, make sure you have set the
//! OPENROUTER_API_KEY environment variable.

use code_forge::ChatEngine;
use std::env;
use futures::StreamExt;

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

/// This test will test connection ask a question and get a response and verify the response
#[tokio::test]
async fn test_chat_engine_response() {
    let mut chat_engine = ChatEngine::new(
        "You are a test assistant.".to_string(),
        "Test connection".to_string()
    );

    match chat_engine.test_connection().await {
        Ok(_) => {
            let mut response = chat_engine.process_message("who is PM of India".to_string()).await;
            let mut final_response = String::new();
            while let Some(response_part) = response.next().await {
                final_response.push_str(&response_part);
            }
            println!("Response: {}", final_response);
            assert_eq!(
                !final_response.is_empty(),
                true,
                "Expected response from chat engine"
            );
            println!("✅ Successfully received response from chat engine");
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
