use std::path::PathBuf;

use anyhow::{Context, Result};
use forge_domain::FileReadService;

use super::Service;

struct Live;

impl Service {
    pub fn file_read_service() -> impl FileReadService {
        Live {}
    }
}

#[async_trait::async_trait]
impl FileReadService for Live {
    async fn read(&self, path: PathBuf) -> Result<String> {
        Ok(tokio::fs::read_to_string(path.clone())
            .await
            .with_context(|| format!("Failed to read file: {}", path.display()))?)
    }
}
