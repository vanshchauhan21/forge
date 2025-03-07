use anyhow::{Context as _, Result};
use derive_builder::Builder;
use forge_domain::{
    self, ChatCompletionMessage, Context as ChatContext, Model, ModelId, Parameters, Provider,
    ProviderService, ResultStream,
};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{Client, Url};
use reqwest_eventsource::{Event, RequestBuilderExt};
use tokio_stream::StreamExt;
use tracing::debug;

use super::model::{ListModelResponse, OpenRouterModel};
use super::parameters::ParameterResponse;
use super::request::OpenRouterRequest;
use super::response::OpenRouterResponse;
use crate::open_router::transformers::{ProviderPipeline, Transformer};

#[derive(Clone, Builder)]
pub struct OpenRouter {
    client: Client,
    provider: Provider,
}

impl OpenRouter {
    pub fn builder() -> OpenRouterBuilder {
        OpenRouterBuilder::default()
    }

    fn url(&self, path: &str) -> anyhow::Result<Url> {
        // Validate the path doesn't contain certain patterns
        if path.contains("://") || path.contains("..") {
            anyhow::bail!("Invalid path: Contains forbidden patterns");
        }

        // Remove leading slash to avoid double slashes
        let path = path.trim_start_matches('/');

        self.provider.to_base_url().join(path).with_context(|| {
            format!(
                "Failed to append {} to base URL: {}",
                path,
                self.provider.to_base_url()
            )
        })
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(ref api_key) = self.provider.key() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
            );
        }
        headers.insert("X-Title", HeaderValue::from_static("code-forge"));
        headers
    }

    // Fetches parameters from the API and returns the raw text response
    async fn fetch_parameters(&self, path: &str) -> Result<String> {
        let url = self.url(path)?;
        let text = self
            .client
            .get(url)
            .headers(self.headers())
            .send()
            .await?
            .error_for_status()
            .with_context(|| "Failed to complete parameter request")?
            .text()
            .await?;

        Ok(text)
    }
}

#[async_trait::async_trait]
impl ProviderService for OpenRouter {
    async fn chat(
        &self,
        model_id: &ModelId,
        request: ChatContext,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        let mut request = OpenRouterRequest::from(request)
            .model(model_id.clone())
            .stream(true);
        request = ProviderPipeline::new(&self.provider).transform(request);

        let url = self.url("chat/completions")?;
        debug!(url = %url, model = %model_id, "Connecting to OpenRouter API");
        let es = self
            .client
            .post(url)
            .headers(self.headers())
            .json(&request)
            .eventsource()?;

        let stream = es
            .take_while(|message| !matches!(message, Err(reqwest_eventsource::Error::StreamEnded)))
            .then(|event| async {
                match event {
                    Ok(event) => match event {
                        Event::Open => None,
                        Event::Message(event) if ["[DONE]", ""].contains(&event.data.as_str()) => {
                            None
                        }
                        Event::Message(event) => Some(
                            serde_json::from_str::<OpenRouterResponse>(&event.data)
                                .with_context(|| "Failed to parse OpenRouter response")
                                .and_then(|message| {
                                    ChatCompletionMessage::try_from(message.clone())
                                        .with_context(|| "Failed to create completion message")
                                }),
                        ),
                    },
                    Err(reqwest_eventsource::Error::StreamEnded) => None,
                    Err(reqwest_eventsource::Error::InvalidStatusCode(_, response)) => Some(
                        response
                            .json::<OpenRouterResponse>()
                            .await
                            .with_context(|| "Failed to parse OpenRouter response")
                            .and_then(|message| {
                                ChatCompletionMessage::try_from(message.clone())
                                    .with_context(|| "Failed to create completion message")
                            })
                            .with_context(|| "Failed with invalid status code"),
                    ),
                    Err(reqwest_eventsource::Error::InvalidContentType(_, response)) => Some(
                        response
                            .json::<OpenRouterResponse>()
                            .await
                            .with_context(|| "Failed to parse OpenRouter response")
                            .and_then(|message| {
                                ChatCompletionMessage::try_from(message.clone())
                                    .with_context(|| "Failed to create completion message")
                            })
                            .with_context(|| "Failed with invalid content type"),
                    ),
                    Err(err) => Some(Err(err.into())),
                }
            });

        Ok(Box::pin(stream.filter_map(|x| x)))
    }

    async fn models(&self) -> Result<Vec<Model>> {
        let response = self
            .client
            .get(self.url("models")?)
            .headers(self.headers())
            .send()
            .await?
            .error_for_status()
            .with_context(|| "Failed because of a non 200 status code".to_string())?
            .text()
            .await?;
        if self.provider.is_open_router() | self.provider.is_antinomy() {
            let data: Vec<OpenRouterModel> = serde_json::from_str(&response)?;
            Ok(data.into_iter().map(Into::into).collect())
        } else {
            // TODO: This could fail for some providers
            let data: ListModelResponse = serde_json::from_str(&response)?;
            Ok(data.data.into_iter().map(Into::into).collect())
        }
    }

    async fn parameters(&self, model: &ModelId) -> Result<Parameters> {
        if self.provider.is_open_router() | self.provider.is_antinomy() {
            // For Eg: https://openrouter.ai/api/v1/parameters/google/gemini-pro-1.5-exp
            let path = if self.provider.is_open_router() {
                format!("parameters/{}", model)
            } else {
                format!("model/{}/parameters", model)
            };

            let text = self.fetch_parameters(&path).await?;

            let response: ParameterResponse = serde_json::from_str(&text)
                .with_context(|| "Failed to parse parameter response".to_string())?;

            Ok(Parameters {
                tool_supported: response
                    .data
                    .supported_parameters
                    .iter()
                    .flat_map(|parameter| parameter.iter())
                    .any(|parameter| parameter == "tools"),
            })
        } else if self.provider.is_open_ai() {
            return Ok(Parameters { tool_supported: true });
        } else {
            return Ok(Parameters { tool_supported: false });
        }
    }
}

impl From<OpenRouterModel> for Model {
    fn from(value: OpenRouterModel) -> Self {
        Model {
            id: value.id,
            name: value.name,
            description: value.description,
            context_length: Some(value.context_length),
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
