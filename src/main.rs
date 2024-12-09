mod cause;
mod error;

use tower_http::services::ServeDir;

use axum::{self, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().nest_service("/assets", ServeDir::new("assets"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
