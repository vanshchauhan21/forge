use super::error::{Error, Result};
use super::provider::{Provider, InnerProvider};
use async_openai::{config::Config, types::*, Client};
use futures::stream::Stream;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

#[derive(Debug, Clone)]
struct OpenRouterConfig {
    api_key: String,
    base_url: Option<String>,
}

impl Config for OpenRouterConfig {
    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn api_base(&self) -> &str {
        self.base_url
            .as_ref()
            .map(|a| a.as_str())
            .unwrap_or("https://openrouter.ai/api/v1")
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
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
pub struct OpenRouter {
    client: Client<OpenRouterConfig>,
    model: String,
}

fn new_message(role: Role, input: &str) -> Result<ChatCompletionRequestMessage> {
    Ok(ChatCompletionRequestMessageArgs::default()
        .role(role)
        .content(input)
        .build()?)
}

impl OpenRouter {
    fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let config = OpenRouterConfig { api_key, base_url };

        let client = Client::with_config(config);

        Self {
            client,
            model: model.unwrap_or("openai/gpt-4o-mini".to_string()),
        }
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

impl InnerProvider for OpenRouter {
    fn name(&self) -> &'static str {
        "Open Router"
    }
    /// Test the connection to OpenRouter
    async fn test(&self) -> Result<bool> {
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

    /// Get a streaming response from OpenRouter
    async fn prompt(&self, input: String) -> Result<Box<dyn Stream<Item = Result<String>>>> {
        let client = self.client.clone();
        let request = self.prompt_request(input)?;
        // Spawn task to handle streaming response

        let stream = client.chat().create_stream(request).await?;

        Ok(Box::new(stream.map(|a| match a {
            Ok(response) => {
                if let Some(ref delta) = response.choices[0].delta.content {
                    Ok(delta.to_string())
                } else {
                    Err(Error::empty_response("OpenAI"))
                }
            }
            Err(e) => Err(e.into()),
        })))
    }
}

impl Provider<OpenRouter> {
    pub fn open_router(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        Provider::new(OpenRouter::new(api_key, model, base_url))
    }
}
