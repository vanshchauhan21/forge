use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::Sse;
use axum::routing::get;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::app::App;
use crate::completion::File;
use crate::{EventStream, Result};

pub struct Server {
    state: Arc<App>,
}

impl Default for Server {
    fn default() -> Self {
        Self { state: Arc::new(App::new(".")) }
    }
}

impl Server {
    pub async fn launch(self) -> Result<()> {
        tracing_subscriber::fmt().init();

        if dotenv::dotenv().is_ok() {
            info!("Loaded .env file");
        }

        // Setup HTTP server
        let app = Router::new()
            .route("/conversation", get(conversation_handler))
            .route("/completions", get(completions_handler))
            .route("/health", get(health_handler))
            .layer(CorsLayer::new().allow_origin(Any))
            .with_state(self.state.clone());

        // Spawn HTTP server
        let server = tokio::spawn(async move {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
                .await
                .unwrap();
            info!("Server running on http://127.0.0.1:3000");
            axum::serve(listener, app).await.unwrap();
        });

        // Wait for server to complete (though it runs indefinitely)
        let _ = server.await;

        Ok(())
    }
}

async fn completions_handler(State(state): State<Arc<App>>) -> axum::Json<Vec<File>> {
    let completions = state.completion.list().await;
    axum::Json(completions)
}

#[axum::debug_handler]
async fn conversation_handler(State(state): State<Arc<App>>) -> Sse<EventStream> {
    Sse::new(state.engine.as_stream().await)
}

async fn health_handler() -> axum::response::Response {
    axum::response::Response::builder()
        .status(200)
        .body(axum::body::Body::empty())
        .unwrap()
}
