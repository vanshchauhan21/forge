use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use tokio_stream::StreamExt;

use super::chat_request::ChatRequest;
use super::chat_response::ChatResponse;
use super::model_response::ListModelResponse;
use crate::error::Result;
use crate::provider::{InnerProvider, Provider};
use crate::{Error, ProviderError, Request, Response, ResultStream};

const DEFAULT_MODEL: &str = "openai/gpt-4o-mini";
const PROVIDER_NAME: &str = "Open Router";

#[derive(Debug, Clone)]
struct Config {
    api_key: String,
    base_url: Option<String>,
}

impl Config {
    fn api_base(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://openrouter.ai/api/v1")
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
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
}

#[derive(Clone)]
struct OpenRouter {
    client: Client,
    config: Config,
    model: String,
}

impl OpenRouter {
    fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let config = Config { api_key, base_url };

        let client = Client::builder().build().unwrap();

        Self {
            client,
            config,
            model: model.unwrap_or(DEFAULT_MODEL.to_string()),
        }
    }
}

#[async_trait::async_trait]
impl InnerProvider for OpenRouter {
    type Request = crate::model::Request;
    type Response = crate::model::Response;
    type Error = Error;

    async fn chat(&self, request: Self::Request) -> ResultStream<Self::Response, Self::Error> {
        let mut request = ChatRequest::from(request);
        request.model = self.model.clone();
        request.stream = Some(true);
        let request = serde_json::to_string(&request)?;

        let rb = self
            .client
            .post(self.config.url("/chat/completions"))
            .headers(self.config.headers())
            .body(request);

        let es = EventSource::new(rb).unwrap();

        let stream = es
            .filter_map(|event| match event {
                Ok(ref event) => match event {
                    Event::Open => None,
                    Event::Message(event) => {
                        // Ignoring wasteful events
                        if ["[DONE]", ""].contains(&event.data.as_str()) {
                            return None;
                        }

                        Some(match serde_json::from_str::<ChatResponse>(&event.data) {
                            Ok(response) => crate::Response::try_from(response),
                            Err(_) => {
                                let value: serde_json::Value =
                                    serde_json::from_str(&event.data).unwrap();
                                Err(Error::Provider {
                                    provider: PROVIDER_NAME.to_string(),
                                    error: ProviderError::UpstreamError(value),
                                })
                            }
                        })
                    }
                },
                Err(err) => Some(Err(err.into())),
            })
            .take_while(|message| {
                !matches!(
                    message,
                    Err(Error::EventSource(reqwest_eventsource::Error::StreamEnded))
                )
            });

        Ok(Box::pin(Box::new(stream)))
    }

    async fn models(&self) -> Result<Vec<String>> {
        let text = self
            .client
            .get(self.config.url("/models"))
            .headers(self.config.headers())
            .send()
            .await?
            .text()
            .await?;

        let response: ListModelResponse = serde_json::from_str(&text)?;

        Ok(response
            .data
            .iter()
            .map(|r| r.name.clone())
            .collect::<Vec<String>>())
    }
}

impl Provider<Request, Response, Error> {
    pub fn open_router(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        Provider::new(OpenRouter::new(api_key, model, base_url))
    }
}
