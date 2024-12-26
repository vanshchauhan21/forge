use forge_server::{Result, API};

#[tokio::main]
async fn main() -> Result<()> {
    API::build().await.launch().await
}
