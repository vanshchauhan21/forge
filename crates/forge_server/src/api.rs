use std::sync::Arc;

const SERVER_PORT: u16 = 8080;

use axum::extract::{Json, State};
use axum::response::sse::{Event, Sse};
use axum::routing::{get, post};
use axum::Router;
use forge_provider::{Model, Request};
use forge_tool::Tool;
use tokio_stream::{Stream, StreamExt};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::completion::File;
use crate::conversation::{ChatRequest, Conversation};
use crate::Result;

#[derive(serde::Serialize)]
struct ModelsResponse {
    models: Vec<Model>,
}

#[derive(serde::Serialize)]
struct CompletionResponse {
    files: Vec<File>,
}

pub struct API {
    // TODO: rename Conversation to Server and drop Server
    state: Arc<Conversation>,
}

impl Default for API {
    fn default() -> Self {
        dotenv::dotenv().ok();
        let api_key = std::env::var("FORGE_KEY").expect("FORGE_KEY must be set");
        Self { state: Arc::new(Conversation::new(".", api_key)) }
    }
}

impl API {
    pub async fn launch(self) -> Result<()> {
        tracing_subscriber::fmt().init();

        if dotenv::dotenv().is_ok() {
            info!("Loaded .env file");
        }

        // Setup HTTP server
        let app = Router::new()
            .route("/conversation", post(conversation_handler))
            .route("/completions", get(completions_handler))
            .route("/health", get(health_handler))
            .route("/tools", get(tools_handler))
            .route("/models", get(models_handler))
            .route("/context", get(context_handler))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods([
                        axum::http::Method::GET,
                        axum::http::Method::POST,
                        axum::http::Method::OPTIONS,
                    ])
                    .allow_headers([
                        axum::http::header::CONTENT_TYPE,
                        axum::http::header::AUTHORIZATION,
                    ]),
            )
            .with_state(self.state.clone());

        // Spawn HTTP server
        let server = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{SERVER_PORT}"))
                .await
                .unwrap();
            info!("Server running on http://127.0.0.1:{SERVER_PORT}");
            axum::serve(listener, app).await.unwrap();
        });

        // Wait for server to complete (though it runs indefinitely)
        let _ = server.await;

        Ok(())
    }
}

async fn completions_handler(
    State(state): State<Arc<Conversation>>,
) -> axum::Json<CompletionResponse> {
    let completions = state
        .completions()
        .await
        .expect("Failed to get completions");
    axum::Json(CompletionResponse { files: completions })
}

#[axum::debug_handler]
async fn conversation_handler(
    State(state): State<Arc<Conversation>>,
    Json(request): Json<ChatRequest>,
) -> Sse<impl Stream<Item = std::result::Result<Event, std::convert::Infallible>>> {
    let stream = state
        .chat(request)
        .await
        .expect("Engine failed to respond with a chat message");
    Sse::new(stream.map(|action| {
        let data = serde_json::to_string(&action).expect("Failed to serialize action");
        Ok(Event::default().data(data))
    }))
}

#[axum::debug_handler]
async fn tools_handler(State(state): State<Arc<Conversation>>) -> Json<Vec<Tool>> {
    let tools = state.tools();
    Json(tools)
}

async fn health_handler() -> axum::response::Response {
    axum::response::Response::builder()
        .status(200)
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn models_handler(State(state): State<Arc<Conversation>>) -> Json<ModelsResponse> {
    let models = state.models().await.unwrap_or_default();
    Json(ModelsResponse { models })
}

async fn context_handler(State(state): State<Arc<Conversation>>) -> Json<Request> {
    let request = state.context();
    Json(request)
}
