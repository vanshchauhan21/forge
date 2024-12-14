use std::env;
use async_openai::{
    types::{
        ChatCompletionRequestMessage,
        CreateChatCompletionRequest,
        Role,
        ChatCompletionRequestMessageArgs,
    },
    Client,
};
use futures::stream::Stream;
use futures::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct Agent {
    system_prompt: String,
    user_prompt: String,
    client: Client<async_openai::config::OpenAIConfig>,
    messages: Vec<ChatCompletionRequestMessage>,
}

impl Agent {
    pub fn new(system: String, user: String) -> Self {
        let client = Client::new();

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

    /// Get a streaming response from OpenAI
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
            model: "gpt-3.5-turbo".to_string(),
            messages: self.messages.clone(),
            temperature: Some(0.7),
            stream: Some(true),
            max_tokens: Some(1000),
            ..Default::default()
        };

        let client = self.client.clone();
        
        // Spawn task to handle streaming response
        tokio::spawn(async move {
            let mut stream = client.chat().create_stream(request).await.unwrap();
            let mut current_content = String::new();

            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        if let Some(delta) = &response.choices[0].delta.content {
                            current_content.push_str(delta);
                            let _ = tx.send(delta.clone()).await;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Error: {}", e)).await;
                        break;
                    }
                }
            }

            // Store assistant's message in history
            if !current_content.is_empty() {
                let _ = tx.send("\n".to_string()).await;
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

    pub async fn process_message(&mut self, input: String) -> impl Stream<Item = String> + Unpin {
        self.agent.stream_response(input).await
    }
}
