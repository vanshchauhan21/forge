use forge_domain::{ChatCompletionMessage, Context, Model, ModelId, Parameters, ResultStream};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use tokio_stream::StreamExt;

use super::model::{ListModelResponse, OpenRouterModel};
use super::request::OpenRouterRequest;
use super::response::OpenRouterResponse;
use super::ParameterResponse;
use crate::error::Result;
use crate::provider::ProviderService;
use crate::{Error, Live, ProviderError, Service};

const PROVIDER_NAME: &str = "Open Router";

#[derive(Debug, Clone)]
struct Config {
    api_key: String,
}

impl Config {
    fn api_base(&self) -> &str {
        "https://openrouter.ai/api/v1"
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );
        headers.insert("X-Title", HeaderValue::from_static("Code Forge"));
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
}

impl OpenRouter {
    fn new(api_key: impl ToString) -> Self {
        let config = Config { api_key: api_key.to_string() };

        let client = Client::builder().build().unwrap();

        Self { client, config }
    }
}

#[async_trait::async_trait]
impl ProviderService for OpenRouter {
    async fn chat(&self, request: Context) -> ResultStream<ChatCompletionMessage, Error> {
        let mut request = OpenRouterRequest::from(request);
        request.stream = Some(true);
        let request = serde_json::to_string_pretty(&request)?;
        println!("{}", request);

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
                        // TODO: print should happen only in debug mode
                        println!("{}", &event.data);
                        // Ignoring wasteful events
                        if ["[DONE]", ""].contains(&event.data.as_str()) {
                            return None;
                        }

                        Some(
                            match serde_json::from_str::<OpenRouterResponse>(&event.data) {
                                Ok(response) => ChatCompletionMessage::try_from(response),
                                Err(_) => {
                                    let value: serde_json::Value =
                                        serde_json::from_str(&event.data).unwrap();
                                    Err(Error::Provider {
                                        provider: PROVIDER_NAME.to_string(),
                                        error: ProviderError::UpstreamError(value),
                                    })
                                }
                            },
                        )
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

    async fn models(&self) -> Result<Vec<Model>> {
        let text = self
            .client
            .get(self.config.url("/models"))
            .headers(self.config.headers())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let response: ListModelResponse = serde_json::from_str(&text)?;

        Ok(response
            .data
            .iter()
            .map(|r| r.clone().into())
            .collect::<Vec<Model>>())
    }

    async fn parameters(&self, model: &ModelId) -> Result<Parameters> {
        // https://openrouter.ai/api/v1/parameters/google/gemini-pro-1.5-exp
        let path = format!("/parameters/{}", model.as_str());
        let text = self
            .client
            .get(self.config.url(&path))
            .headers(self.config.headers())
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let response: ParameterResponse = serde_json::from_str(&text)?;

        Ok(Parameters {
            tool_supported: response
                .data
                .supported_parameters
                .iter()
                .flat_map(|parameter| parameter.iter())
                .any(|parameter| parameter == "tools"),
        })
    }
}

impl Service {
    pub fn open_router(api_key: impl ToString) -> impl ProviderService {
        Live::new(OpenRouter::new(api_key))
    }
}

impl From<OpenRouterModel> for Model {
    fn from(value: OpenRouterModel) -> Self {
        Model {
            id: value.id,
            name: value.name,
            description: value.description,
        }
    }
}
