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

#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use super::*;

    #[derive(Default)]
    pub struct TestFileReadService(HashMap<String, String>);

    impl TestFileReadService {
        pub fn add(mut self, path: impl ToString, content: impl ToString) -> Self {
            self.0.insert(path.to_string(), content.to_string());
            self
        }
    }

    #[async_trait::async_trait]
    impl FileReadService for TestFileReadService {
        async fn read(&self, path: PathBuf) -> Result<String> {
            let path_str = path.to_string_lossy().to_string();
            self.0.get(&path_str).cloned().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found: {}", path_str),
                )
                .into()
            })
        }
    }
}
