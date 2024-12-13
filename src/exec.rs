use crate::error::Result;
use axum::body::{to_bytes, Body};
use axum::http::HeaderValue;
use axum::{
    http::{Request, Response},
    response::IntoResponse,
};
use derive_more::Debug;
use rig::agent::Agent;
use rig::completion::Prompt;
use rig::providers::openai::{self, CompletionModel};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Exec {}

impl Exec {
    pub fn new() -> Self {
        Exec {}
    }

    pub async fn execute(&self, request: Request<Body>) -> Result<Response<Body>> {
        let (_, body) = request.into_parts();
        let bytes = to_bytes(body, usize::MAX).await?;
        let prompt = serde_json::from_slice(bytes.as_ref())?;
        let action = LLMAgent::default().execute(prompt).await?;
        let mut response = Response::new(Body::from(serde_json::to_vec(&action)?).into());
        response
            .headers_mut()
            .append("Content-Type", HeaderValue::from_static("application/json"));
        Ok(response)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum Action {
    Prompt(String),
}

impl Action {
    async fn from_request(request: Request<Body>) -> Result<Self> {
        let (_, body) = request.into_parts();
        let bytes = to_bytes(body, usize::MAX).await?;
        Ok(serde_json::from_slice(bytes.as_ref())?)
    }
}

#[derive(Debug, Serialize)]
enum Command {
    Submit(String),
}

impl IntoResponse for Command {
    fn into_response(self) -> axum::response::Response {
        match serde_json::to_vec(&self) {
            Ok(body) => axum::http::Response::new(Body::from(body)).into_response(),
            Err(_) => todo!(),
        }
    }
}

struct LLMAgent {
    agent: Agent<CompletionModel>,
}

impl LLMAgent {
    fn default() -> Self {
        // A place to initialize our LLM agent
        let agent = openai::Client::from_env().agent("gpt-4o-mini").build();
        LLMAgent { agent }
    }

    async fn execute(&self, action: Action) -> Result<Command> {
        match action {
            Action::Prompt(message) => {
                let response = self.agent.prompt(message.as_str()).await?;
                Ok(Command::Submit(response))
            }
        }
    }
}
