use super::error::{Error, Result};
use async_openai::{config::Config, types::*, Client};
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
    pub fn new(api_key: String, model: String, base_url: String) -> Self {
        let config = OpenAIConfig { api_key, base_url };
        let client = Client::with_config(config);
        Self { client, model }
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
                    Err(Error::EmptyResponse("OpenAI".to_string())) // Updated to include provider name
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

pub struct Engine {
    provider: OpenAIProvider,
}

impl Engine {
    pub fn new(key: String, model: String, base_url: String) -> Self {
        Self {
            provider: OpenAIProvider::new(key, model, base_url),
        }
    }

    pub async fn test(&self) -> Result<bool> {
        self.provider.test().await
    }

    pub async fn prompt(&mut self, input: String) -> Result<impl Stream<Item = Result<String>>> {
        let response = self.provider.prompt(input).await?;
        Ok(response)
    }
}
