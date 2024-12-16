use async_openai::{
    config::Config,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs,
        CreateChatCompletionRequest, Role,
    },
    Client,
};
use futures::stream::Stream;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::env;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
const MODEL_NAME: &str = "anthropic/claude-3-sonnet";

#[derive(Clone)]
struct OpenRouterConfig {
    api_key: String,
}

impl Config for OpenRouterConfig {
    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn api_base(&self) -> &str {
        OPENROUTER_BASE_URL
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("X-Title", HeaderValue::from_static("Code Forge Chat"));
        headers
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base(), path)
    }

    fn query(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
}

#[derive(Clone)]
pub struct Agent {
    client: Client<OpenRouterConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
}

impl Agent {
    pub fn new(system: String) -> Self {
        let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
        let config = OpenRouterConfig { api_key };
        let client = Client::with_config(config);

        let messages = vec![ChatCompletionRequestMessageArgs::default()
            .role(Role::System)
            .content(system.clone())
            .build()
            .unwrap()];

        Self { client, messages }
    }

    /// Test the connection to OpenRouter
    pub async fn test_connection(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request = CreateChatCompletionRequest {
            model: MODEL_NAME.to_string(),
            messages: vec![
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content("You are a helpful AI assistant.")
                    .build()
                    .unwrap(),
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content("Respond with 'Connected successfully!' if you receive this message.")
                    .build()
                    .unwrap(),
            ],
            temperature: Some(0.7),
            stream: Some(false),
            max_tokens: Some(50),
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;
        Ok(response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default())
    }

    /// Get a streaming response from OpenRouter
    pub async fn stream_response(&mut self, input: String) -> impl Stream<Item = String> + Unpin {
        let (tx, rx) = mpsc::channel::<String>(100);

        // Add user message to history
        self.messages.push(
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(input)
                .build()
                .unwrap(),
        );

        // Create chat completion request
        let request = CreateChatCompletionRequest {
            model: MODEL_NAME.to_string(),
            messages: self.messages.clone(),
            temperature: Some(0.7),
            stream: Some(true),
            max_tokens: Some(1000),
            ..Default::default()
        };

        let client = self.client.clone();
        let messages = self.messages.clone();

        // Spawn task to handle streaming response
        tokio::spawn(async move {
            match client.chat().create_stream(request).await {
                Ok(mut stream) => {
                    let mut current_content = String::new();

                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(response) => {
                                if let Some(ref delta) = response.choices[0].delta.content {
                                    current_content.push_str(delta);
                                    let _ = tx.send(delta.to_string()).await;
                                }
                            }
                            Err(e) => {
                                eprintln!("Error in stream: {}", e);
                                let _ = tx.send(format!("Error: {}", e)).await;
                                break;
                            }
                        }
                    }

                    // Add assistant's message to history and send newline to signal completion
                    if !current_content.is_empty() {
                        let mut messages = messages;
                        messages.push(
                            ChatCompletionRequestMessageArgs::default()
                                .role(Role::Assistant)
                                .content(current_content)
                                .build()
                                .unwrap(),
                        );
                        let _ = tx.send("\n".to_string()).await;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create stream: {}", e);
                    let _ = tx.send(format!("Error: {}", e)).await;
                }
            }
        });

        ReceiverStream::new(rx)
    }
}

pub struct ChatEngine {
    agent: Agent,
}

impl ChatEngine {
    pub fn new(system_prompt: String) -> Self {
        Self {
            agent: Agent::new(system_prompt),
        }
    }

    pub async fn test_connection(
        &self,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.agent.test_connection().await
    }

    pub async fn process_message(&mut self, input: String) -> impl Stream<Item = String> + Unpin {
        self.agent.stream_response(input).await
    }
}

#[cfg(test)]
mod test {
    //! Integration tests for Code Forge Chat
    //!
    //! These tests verify the connection to OpenRouter and basic functionality
    //! of the chat engine. To run these tests, make sure you have set the
    //! OPENROUTER_API_KEY environment variable.

    use crate::ChatEngine; // Changed from code_forge to crate
    use futures::StreamExt;
    use std::env;

    /// Test the connection to OpenRouter and verify we can access Claude 3 Sonnet
    #[tokio::test]
    async fn test_openrouter_connection() {
        // Verify API key is set
        if env::var("OPENROUTER_API_KEY").is_err() {
            panic!("❌ OPENROUTER_API_KEY environment variable is not set");
        }

        let chat_engine = ChatEngine::new("You are a test assistant.".to_string()); // Removed second argument

        match chat_engine.test_connection().await {
            Ok(response) => {
                assert!(
                    response.contains("Connected successfully"),
                    "Expected connection test response, got: {}",
                    response
                );
                println!("✅ Successfully connected to OpenRouter");
                println!("Response: {}", response);
            }
            Err(e) => {
                panic!(
                    "❌ Failed to connect to OpenRouter: {}\n\
                   \nTroubleshooting steps:\n\
                   1. Verify your API key is valid\n\
                   2. Check if you have access to 'anthropic/claude-3-sonnet-20240229'\n\
                   3. Visit https://openrouter.ai/docs#models for supported models\n\
                   4. Check your internet connection\n",
                    e
                );
            }
        }
    }

    /// Helper function to run the test with proper environment setup
    #[tokio::test]
    async fn test_chat_engine_initialization() {
        let chat_engine = ChatEngine::new("Test system prompt".to_string()); // Removed second argument

        assert!(
            chat_engine.test_connection().await.is_ok(),
            "Chat engine should initialize and connect successfully"
        );
    }

    /// This test will test connection ask a question and get a response and verify the response
    #[tokio::test]
    async fn test_chat_engine_response() {
        let mut chat_engine = ChatEngine::new("You are a test assistant.".to_string());

        match chat_engine.test_connection().await {
            Ok(_) => {
                let mut response = chat_engine
                    .process_message("who is PM of India".to_string())
                    .await;
                let mut final_response = String::new();
                while let Some(response_part) = response.next().await {
                    final_response.push_str(&response_part);
                }
                println!("Response: {}", final_response);
                assert!(
                    !final_response.is_empty(),
                    "Expected response from chat engine"
                );
                println!("✅ Successfully received response from chat engine");
            }
            Err(e) => {
                panic!(
                    "❌ Failed to connect to OpenRouter: {}\n\
                   \nTroubleshooting steps:\n\
                   1. Verify your API key is valid\n\
                   2. Check if you have access to 'anthropic/claude-3-sonnet-20240229'\n\
                   3. Visit https://openrouter.ai/docs#models for supported models\n\
                   4. Check your internet connection\n",
                    e
                );
            }
        }
    }
}
