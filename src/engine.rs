use std::env;
use async_openai::{
    types::{
        ChatCompletionRequestMessage,
        CreateChatCompletionRequest,
        Role,
        ChatCompletionRequestMessageArgs,
    },
    Client, config::Config,
};
use futures::stream::Stream;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

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
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(
            "X-Title",
            HeaderValue::from_static("Code Forge Chat"),
        );
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
    system_prompt: String,
    user_prompt: String,
    client: Client<OpenRouterConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
}

impl Agent {
    pub fn new(system: String, user: String) -> Self {
        let api_key = env::var("OPENROUTER_API_KEY").expect("OPENROUTER_API_KEY must be set");
        let config = OpenRouterConfig { api_key };
        let client = Client::with_config(config);

        let mut messages = Vec::new();
        messages.push(
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(system.clone())
                .build()
                .unwrap()
        );

        Self {
            system_prompt: system,
            user_prompt: user,
            client,
            messages,
        }
    }

    /// Test the connection to OpenRouter
    pub async fn test_connection(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
        Ok(response.choices[0].message.content.clone().unwrap_or_default())
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
                .unwrap()
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
                                .unwrap()
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
    pub fn new(system_prompt: String, user_prompt: String) -> Self {
        Self {
            agent: Agent::new(system_prompt, user_prompt),
        }
    }

    pub async fn test_connection(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.agent.test_connection().await
    }

    pub async fn process_message(&mut self, input: String) -> impl Stream<Item = String> + Unpin {
        self.agent.stream_response(input).await
    }
}
