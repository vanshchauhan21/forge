use anyhow::{Context as _, Result};
use forge_domain::{
    self, ChatCompletionMessage, Context as ChatContext, Model, ModelId, Parameters,
    ProviderService, ResultStream,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use tokio_stream::StreamExt;

use super::model::{ListModelResponse, OpenRouterModel};
use super::parameters::ParameterResponse;
use super::request::OpenRouterRequest;
use super::response::OpenRouterResponse;
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
pub struct OpenRouter {
    client: Client,
    config: Config,
}

impl OpenRouter {
    pub fn new(api_key: impl ToString) -> Self {
        let config = Config { api_key: api_key.to_string() };

        let client = Client::builder().build().unwrap();

        Self { client, config }
    }
}

#[async_trait::async_trait]
impl ProviderService for OpenRouter {
    async fn chat(
        &self,
        model_id: &ModelId,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        let mut request = OpenRouterRequest::from(request);

        // Use the passed model_id
        request.model = Some(model_id.clone());

        request.stream = Some(true);
        let request = serde_json::to_string(&request)?;

        let rb = self
            .client
            .post(self.config.url("/chat/completions"))
            .headers(self.config.headers())
            .body(request);

        let es = EventSource::new(rb).unwrap();

        let stream = es
            .take_while(|message| !matches!(message, Err(reqwest_eventsource::Error::StreamEnded)))
            .filter_map(|event| match event {
                Ok(ref event) => match event {
                    Event::Open => None,
                    Event::Message(event) => {
                        // Ignoring wasteful events
                        if ["[DONE]", ""].contains(&event.data.as_str()) {
                            return None;
                        }

                        let message = serde_json::from_str::<OpenRouterResponse>(&event.data)
                            .with_context(|| "Failed to parse OpenRouter response")
                            .and_then(|message| {
                                Ok(ChatCompletionMessage::try_from(message.clone())?)
                            });
                        Some(message)
                    }
                },
                Err(reqwest_eventsource::Error::StreamEnded) => None,
                Err(err) => Some(Err(err.into())),
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

impl From<OpenRouterModel> for Model {
    fn from(value: OpenRouterModel) -> Self {
        Model {
            id: value.id,
            name: value.name,
            description: value.description,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;

    use super::*;

    #[test]
    fn test_error_deserialization() -> Result<()> {
        let content = serde_json::to_string(&serde_json::json!({
          "error": {
            "message": "This endpoint's maximum context length is 16384 tokens",
            "code": 400
          }
        }))
        .unwrap();
        let message = serde_json::from_str::<OpenRouterResponse>(&content)
            .context("Failed to parse response")?;
        let message = ChatCompletionMessage::try_from(message.clone());

        assert!(message.is_err());
        Ok(())
    }
}
