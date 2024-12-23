use std::convert::Infallible;
use std::path::Path;
use std::sync::Arc;

use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::routing::get;
use axum::Router;
use clap::Parser;
use forge_cli::cli::Cli;
use forge_cli::{Engine, Result};
use futures::stream::{self, Stream};
use futures::StreamExt;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

// Shared state between HTTP server and CLI
#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<String>,
}

async fn conversation_handler(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();

    // Create a stream that emits incrementing numbers every second
    let counter_stream = stream::unfold(0u64, |counter| async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Some((counter, counter + 1))
    });

    // Merge the counter stream with the broadcast receiver
    let combined_stream = stream::select(
        // Convert counter to events
        counter_stream.map(|n| Ok(Event::default().data(n.to_string()))),
        // Original broadcast receiver stream
        stream::unfold(rx, |mut rx| async move {
            match rx.recv().await {
                Ok(msg) => {
                    let event = Event::default().data(msg);
                    Some((Ok(event), rx))
                }
                Err(_) => None,
            }
        }),
    );

    Sse::new(combined_stream)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with level from CLI
    tracing_subscriber::fmt()
        .with_max_level(cli.log_level.clone().unwrap_or_default())
        .init();

    // Create broadcast channel for SSE
    let (tx, _) = broadcast::channel::<String>(100);
    let state = Arc::new(AppState { tx: tx.clone() });

    // Setup HTTP server
    let app = Router::new()
        .route("/conversation", get(conversation_handler))
        .layer(CorsLayer::new().allow_origin(Any))
        .with_state(state.clone());

    // Spawn HTTP server
    let server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .unwrap();
        info!("Server running on http://127.0.0.1:3000");
        axum::serve(listener, app).await.unwrap();
    });

    // Run CLI engine
    let engine = Engine::new(cli, Path::new(".").to_path_buf(), tx.clone());
    engine.launch().await?;

    // Wait for server to complete (though it runs indefinitely)
    let _ = server.await;

    Ok(())
}
