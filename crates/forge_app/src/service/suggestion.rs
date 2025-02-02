use std::path::PathBuf;

use anyhow::Result;
use forge_walker::Walker;
use serde::Serialize;

use super::Service;

#[derive(Serialize)]
pub struct File {
    pub path: String,
    pub is_dir: bool,
}

#[async_trait::async_trait]
pub trait SuggestionService: Send + Sync {
    async fn suggestions(&self) -> Result<Vec<File>>;
}

struct Live {
    path: PathBuf,
}

impl Live {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

#[async_trait::async_trait]
impl SuggestionService for Live {
    async fn suggestions(&self) -> Result<Vec<File>> {
        let cwd = self.path.clone(); // Use the current working directory
        let walker = Walker::max_all().cwd(cwd);

        let files = walker.get().await?;
        Ok(files
            .into_iter()
            .map(|file| File { path: file.path.clone(), is_dir: file.is_dir() })
            .collect())
    }
}

impl Service {
    pub fn completion_service(path: impl Into<PathBuf>) -> impl SuggestionService {
        Live::new(path)
    }
}
