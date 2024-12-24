mod request;
mod response;

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use super::error::Result;
use super::provider::{InnerProvider, Provider};
use crate::{Error, ProviderError, ResultStream};

use request::Request;
use response::Response;

const DEFAULT_MODEL: &str = "openai/gpt-4o-mini";
const PROVIDER_NAME: &str = "Open Router";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Model {
    id: String,
    name: String,
    created: u64,
    description: String,
    context_length: u64,
    architecture: Architecture,
    pricing: Pricing,
    top_provider: TopProvider,
    per_request_limits: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Architecture {
    modality: String,
    tokenizer: String,
    instruct_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct Pricing {
    prompt: String,
    completion: String,
    image: String,
    request: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct TopProvider {
    context_length: Option<u64>,
    max_completion_tokens: Option<u64>,
    is_moderated: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Serialize)]
struct ListModelResponse {
    data: Vec<Model>,
}

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
    #[allow(unused)]
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
    async fn chat(
        &self,
        body: crate::model::Request,
    ) -> Result<ResultStream<crate::model::Response>> {
        let mut new_body = Request::from(body);

        new_body.model = self.model.clone();
        new_body.stream = Some(true);

        let body = serde_json::to_string(&new_body)?;

        tracing::debug!("Request Body: {}", body);

        let rb = self
            .client
            .post(self.config.url("/chat/completions"))
            .headers(self.config.headers())
            .body(body);

        let mut es = EventSource::new(rb).unwrap();

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<crate::Response>>(100);
        while let Some(event) = es.next().await {
            match event {
                Ok(event) => match event {
                    Event::Open => {
                        dbg!("Connection Opened");
                    }
                    Event::Message(event) => {
                        dbg!(&event.event);
                        dbg!(&event.data);

                        if event.data == "[DONE]" {
                            break;
                        }

                        match serde_json::from_str::<Response>(&event.data) {
                            Ok(response) => {
                                let response = crate::Response::try_from(response);
                                tx.send(response).await.unwrap();
                            }
                            Err(_) => {
                                let value: serde_json::Value =
                                    serde_json::from_str(&event.data).unwrap();

                                tx.send(Err(Error::Provider {
                                    provider: PROVIDER_NAME.to_string(),
                                    error: ProviderError::UpstreamError(value),
                                }))
                                .await
                                .unwrap();
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    tx.send(Err(err.into())).await.unwrap();
                    break;
                }
            }
        }

        let processed_stream = ReceiverStream::new(rx);

        Ok(Box::pin(Box::new(processed_stream)))
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

impl Provider {
    pub fn open_router(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        Provider::new(OpenRouter::new(api_key, model, base_url))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn models() -> &'static str {
        include_str!("models.json")
    }

    #[test]
    fn test_de_ser_of_models() {
        let _: ListModelResponse = serde_json::from_str(models()).unwrap();
    }

    #[test]
    fn test_de_ser_of_response() {
        let response = r#"{"id":"gen-1734752897-QSJJJjXmljCFFkUZHtFk","provider":"Anthropic","model":"anthropic/claude-3.5-sonnet","object":"chat.completion","created":1734752897,"choices":[{"logprobs":null,"finish_reason":"end_turn","index":0,"message":{"role":"assistant","content":"I aim to be direct and honest in my interactions: I'm an AI assistant, so I don't experience feelings in the way humans do. I aim to be helpful while being transparent about what I am. How can I assist you today?","refusal":""}}],"usage":{"prompt_tokens":13,"completion_tokens":54,"total_tokens":67}}"#;

        let _: Response = serde_json::from_str(response).unwrap();
    }

    #[tokio::test]
    async fn test_chat() {
        let provider = Provider::new(OpenRouter::new(
            "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".to_string(),
            None,
            None,
        ));

        let result_stream = provider
            .chat(crate::model::Request {
                context: vec![
                    crate::model::AnyMessage::User(crate::model::Message {
                        role: crate::model::Role::User,
                        content: "Hello!".to_string(),
                    }),
                    crate::model::AnyMessage::System(crate::model::Message {
                        role: crate::model::Role::System,
                        content: "If someone says Hello!, always Reply with single word Alo!"
                            .to_string(),
                    }),
                ],
                tools: vec![],
                tool_result: vec![],
            })
            .await
            .unwrap();

        let mut stream = result_stream;

        while let Some(result) = stream.next().await {
            if let Ok(response) = result {
                assert_eq!(response.message.content.trim(), "Alo!");
            }
        }
    }
}
