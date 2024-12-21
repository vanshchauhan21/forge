use http::Extensions;
use reqwest_middleware::{Middleware, Next, Result};
use tracing::info;

pub struct LoggingMiddleware;

#[async_trait::async_trait]
impl Middleware for LoggingMiddleware {
    async fn handle(
        &self,
        req: reqwest_middleware::reqwest::Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<reqwest_middleware::reqwest::Response> {
        info!("Request: {:?}", req);
        let response = next.run(req, extensions).await;
        info!("Response: {:?}", response);
        response
    }
}
