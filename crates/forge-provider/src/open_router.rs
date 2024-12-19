use crate::model::{ContentPart, ListModelResponse, Message, Request, Response, TextContent};

use super::error::Result;
use super::open_ai::Role; // Importing Role
use super::provider::{InnerProvider, Provider};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
 // Importing Stream trait

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

    fn new_message(&self, role: Role, input: &str) -> Message {
        Message {
            role: role.to_string(),
            content: ContentPart::Text(TextContent {
                r#type: "text".to_string(),
                text: input.to_string(),
            }),
            name: None,
        }
    }
}

#[async_trait::async_trait]
impl InnerProvider for OpenRouter {
    fn name(&self) -> &'static str {
        "Open Router"
    }

    async fn chat(&self, mut request: Request) -> Result<Response> {
        request.stream = Some(false);
        Ok(self
            .http_client
            .post(self.config.url("/chat/completions"))
            .headers(self.config.headers())
            .json(&request)
            .send()
            .await?
            .json::<Response>() // Adjusted to use ResponseType
            .await?)
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
