use crate::request::{Request, Response};

use super::error::Result;
use super::provider::{InnerProvider, Provider};
use async_openai::{config, types::*, Client};
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
struct Config {
    api_key: String,
    base_url: Option<String>,
}

impl config::Config for Config {
    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn api_base(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
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
    client: Client<Config>,
    model: String,
    http_client: reqwest::Client,
    config: Config,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Model {
    pub id: String,
    pub name: String,
    pub created: u64,
    pub description: String,
    pub context_length: u64,
    pub architecture: Architecture,
    pub pricing: Pricing,
    pub top_provider: TopProvider,
    pub per_request_limits: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Architecture {
    pub modality: String,
    pub tokenizer: String,
    pub instruct_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Pricing {
    pub prompt: String,
    pub completion: String,
    pub image: String,
    pub request: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct TopProvider {
    pub context_length: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub is_moderated: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
pub struct ListModelResponse {
    pub object: Option<String>,
    pub data: Vec<Model>,
}

// Making new_message public
pub fn new_message(role: Role, input: &str) -> Result<ChatCompletionRequestMessage> {
    Ok(ChatCompletionRequestMessageArgs::default()
        .role(role)
        .content(input)
        .build()?)
}

// Making Role public
pub use async_openai::types::Role;

impl OpenRouter {
    fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let config = Config { api_key, base_url };
        let http_client = reqwest::Client::new();
        let client = Client::with_config(config.clone()).with_http_client(http_client.clone());

        Self {
            client,
            http_client,
            config,
            model: model.unwrap_or("gpt-4o-mini".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl InnerProvider for OpenRouter {
    fn name(&self) -> &'static str {
        "Open Router"
    }

    async fn chat(&self, request: Request) -> Result<Response> {
        let client = self.client.clone();
        let response = client.chat().create(request.into()).await?;
        Ok(response.into())
    }

    async fn models(&self) -> Result<Vec<String>> {
        Ok(self
            .client
            .models()
            .list()
            .await?
            .data
            .iter()
            .map(|r| r.id.clone())
            .collect())
    }
}

impl Provider {
    pub fn open_ai(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        Provider::new(OpenRouter::new(api_key, model, base_url))
    }
}

impl From<Request> for CreateChatCompletionRequest {
    fn from(value: Request) -> Self {
        todo!()
    }
}

impl From<CreateChatCompletionResponse> for Response {
    fn from(value: CreateChatCompletionResponse) -> Self {
        todo!()
    }
}
