use std::path::PathBuf;

use anyhow::Result;

use super::Service;

#[async_trait::async_trait]
pub trait FileReadService: Send + Sync {
    async fn read(&self, path: PathBuf) -> Result<String>;
}

impl Service {
    pub fn file_read_service() -> impl FileReadService {
        Live {}
    }
}

struct Live;

#[async_trait::async_trait]
impl FileReadService for Live {
    async fn read(&self, path: PathBuf) -> Result<String> {
        Ok(tokio::fs::read_to_string(path).await?)
    }
}
