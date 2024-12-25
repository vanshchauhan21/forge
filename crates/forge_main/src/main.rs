use forge_server::{Result, API};

#[tokio::main]
async fn main() -> Result<()> {
    API::default().launch().await
}
