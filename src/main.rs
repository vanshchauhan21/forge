mod api;
mod cause;
mod error;

use api::API;
use tower_http::services::ServeDir;

use axum::{self, Router};

#[tokio::main]
async fn main() {
    let api = API::new();
    let app = Router::new()
        .nest("/api", api.into_router())
        .nest_service("/assets", ServeDir::new("assets"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
