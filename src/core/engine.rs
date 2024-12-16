use super::error::{Error, Result};
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
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

#[derive(Clone)]
struct OpenAIConfig {
    api_key: String,
    base_url: String,
}

impl Config for OpenAIConfig {
    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn api_base(&self) -> &str {
        &self.base_url
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("X-Title", HeaderValue::from_static("Tailcall"));
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
pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

fn new_message(role: Role, input: &str) -> Result<ChatCompletionRequestMessage> {
    Ok(ChatCompletionRequestMessageArgs::default()
        .role(role)
        .content(input)
        .build()?)
}

impl OpenAIProvider {
    const BASE_URL: &str = "https://api.openai.com/v1";
    const MODEL: &str = "gpt-4o";

    pub fn new(api_key: String) -> Self {
        let config = OpenAIConfig {
            api_key,
            base_url: Self::BASE_URL.to_string(),
        };
        let client = Client::with_config(config);
        Self {
            client,
            model: Self::MODEL.to_string(),
        }
    }

    /// Test the connection to OpenRouter
    pub async fn test(&self) -> Result<bool> {
        let request = self.test_request()?;

        let response = self.client.chat().create(request).await?;

        let ok = response.choices.iter().any(|c| {
            c.message
                .content
                .iter()
                .any(|c| c == "Connected successfully!")
        });

        Ok(ok)
    }

    fn test_request(&self) -> Result<CreateChatCompletionRequest> {
        Ok(CreateChatCompletionRequest {
            model: self.model.to_string(),
            messages: vec![
                new_message(Role::System, "You are a helpful AI assistant.")?,
                new_message(
                    Role::User,
                    "Respond with 'Connected successfully!' if you receive this message.",
                )?,
            ],
            temperature: Some(0.0),
            stream: Some(false),
            max_tokens: Some(50),
            ..Default::default()
        })
    }

    /// Get a streaming response from OpenRouter
    pub async fn prompt(&self, input: String) -> Result<impl Stream<Item = Result<String>>> {
        let client = self.client.clone();
        let request = self.prompt_request(input)?;
        // Spawn task to handle streaming response

        let stream = client.chat().create_stream(request).await?;

        Ok(stream.map(|a| match a {
            Ok(response) => {
                if let Some(ref delta) = response.choices[0].delta.content {
                    Ok(delta.to_string())
                } else {
                    Err(Error::EmptyResponse)
                }
            }
            Err(e) => Err(e.into()),
        }))
    }

    fn prompt_request(&self, input: String) -> Result<CreateChatCompletionRequest> {
        Ok(CreateChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![new_message(Role::User, &input)?],

            // TODO: Make temperature configurable
            temperature: Some(0.7),
            stream: Some(true),
            ..Default::default()
        })
    }
}

pub struct ChatEngine {
    provider: OpenAIProvider,
}

impl ChatEngine {
    pub fn new(system_prompt: String) -> Self {
        Self {
            provider: OpenAIProvider::new(system_prompt),
        }
    }

    pub async fn test(&self) -> Result<bool> {
        self.provider.test().await
    }

    pub async fn prompt(&mut self, input: String) -> Result<impl Stream<Item = Result<String>>> {
        self.provider.prompt(input).await
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

        match chat_engine.test().await {
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
            chat_engine.test().await.is_ok(),
            "Chat engine should initialize and connect successfully"
        );
    }

    /// This test will test connection ask a question and get a response and verify the response
    #[tokio::test]
    async fn test_chat_engine_response() {
        let mut chat_engine = ChatEngine::new("You are a test assistant.".to_string());

        match chat_engine.test().await {
            Ok(_) => {
                let mut response = chat_engine.prompt("who is PM of India".to_string()).await;
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
