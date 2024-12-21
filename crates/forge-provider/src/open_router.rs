use forge_tool::Tool;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use super::error::Result;
use super::provider::{InnerProvider, Provider};
use crate::model::{AnyMessage, Assistant, Request, Response, Role, System, User};

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
    pub data: Vec<Model>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct OpenRouterRequest {
    pub messages: Option<Vec<Message>>,
    pub prompt: Option<String>,
    pub model: Option<String>,
    pub response_format: Option<ResponseFormat>,
    pub stop: Option<Vec<String>>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<OpenRouterTool>>,
    pub tool_choice: Option<ToolChoice>,
    pub seed: Option<u32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub repetition_penalty: Option<f32>,
    pub logit_bias: Option<std::collections::HashMap<u32, f32>>,
    pub top_logprobs: Option<u32>,
    pub min_p: Option<f32>,
    pub top_a: Option<f32>,
    pub prediction: Option<Prediction>,
    pub transforms: Option<Vec<String>>,
    pub models: Option<Vec<String>>,
    pub route: Option<String>,
    pub provider: Option<ProviderPreferences>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TextContent {
    // TODO: could be an enum
    pub r#type: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageContentPart {
    pub r#type: String,
    pub image_url: ImageUrl,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageUrl {
    pub url: String,
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ContentPart {
    Text(TextContent),
    Image(ImageContentPart),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: ContentPart,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionDescription {
    pub description: Option<String>,
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenRouterTool {
    // TODO: should be an enum
    pub r#type: String,
    pub function: FunctionDescription,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ToolChoice {
    None,
    Auto,
    Function { name: String },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseFormat {
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Prediction {
    pub r#type: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenRouterResponse {
    pub status: String,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl From<Tool> for OpenRouterTool {
    fn from(value: Tool) -> Self {
        OpenRouterTool {
            r#type: "function".to_string(),
            function: FunctionDescription {
                description: Some(value.description),
                name: value.id.into_string(),
                parameters: value.input_schema,
            },
        }
    }
}

// TODO: fix the names.
impl From<AnyMessage> for Message {
    fn from(value: AnyMessage) -> Self {
        match value {
            AnyMessage::Assistant(assistant) => Message {
                role: Assistant::name(),
                content: ContentPart::Text(TextContent {
                    r#type: "text".to_string(),
                    text: assistant.content,
                }),
                name: None,
            },
            AnyMessage::System(sys) => Message {
                role: System::name(),
                content: ContentPart::Text(TextContent {
                    r#type: "text".to_string(),
                    text: sys.content,
                }),
                name: None,
            },
            AnyMessage::User(usr) => Message {
                role: User::name(),
                content: ContentPart::Text(TextContent {
                    r#type: "text".to_string(),
                    text: usr.content,
                }),
                name: None,
            },
        }
    }
}

impl From<Request> for OpenRouterRequest {
    fn from(value: Request) -> Self {
        OpenRouterRequest {
            messages: Some(
                value
                    .context
                    .into_iter()
                    .map(Message::from)
                    .collect::<Vec<_>>(),
            ),
            tools: Some(
                value
                    .tools
                    .into_iter()
                    .map(OpenRouterTool::from)
                    .collect::<Vec<_>>(),
            ),
            ..Default::default()
        }
    }
}

impl From<OpenRouterResponse> for Response {
    fn from(value: OpenRouterResponse) -> Self {
        todo!()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProviderPreferences {
    // Define fields as necessary
}

#[derive(Debug, Clone)]
struct Config {
    api_key: String,
    base_url: Option<String>,
}

impl Config {
    fn api_key(&self) -> &str {
        &self.api_key
    }

    fn api_base(&self) -> &str {
        self.base_url
            .as_deref()
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
    http_client: reqwest::Client,
    config: Config,
    model: String,
}

impl OpenRouter {
    fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let config = Config { api_key, base_url };
        let http_client = reqwest::Client::new();

        Self {
            http_client,
            config,
            model: model.unwrap_or("openai/gpt-4o-mini".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl InnerProvider for OpenRouter {
    fn name(&self) -> &'static str {
        "Open Router"
    }

    async fn chat(&self, request: Request) -> Result<Response> {
        let open_router_request = OpenRouterRequest::from(request);
        let response = self
            .http_client
            .post(self.config.url("/chat/completions"))
            .headers(self.config.headers())
            .json(&open_router_request)
            .send()
            .await?
            .json::<OpenRouterResponse>() // Adjusted to use ResponseType
            .await?;

        Ok(response.into())
    }

    async fn models(&self) -> Result<Vec<String>> {
        Ok(self
            .http_client
            .get(self.config.url("/models"))
            .headers(self.config.headers())
            .send()
            .await?
            .json::<ListModelResponse>()
            .await?
            .data
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>())
    }
}

impl Provider {
    pub fn open_router(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        Provider::new(OpenRouter::new(api_key, model, base_url))
    }
}

#[cfg(test)]
mod test {
    use crate::open_router::ListModelResponse;

    fn models() -> &'static str {
        include_str!("models.json")
    }

    #[test]
    fn test_ser_of_models() {
        let response: Result<ListModelResponse, serde_json::Error> = serde_json::from_str(models());
        assert!(response.is_ok())
    }
}
