use super::Service;
use crate::Result;

#[async_trait::async_trait]
pub trait FileReadService: Send + Sync {
    async fn read(&self, path: String) -> Result<String>;
}

impl Service {
    pub fn file_read_service() -> impl FileReadService {
        Live {}
    }
}

struct Live;

#[async_trait::async_trait]
impl FileReadService for Live {
    async fn read(&self, path: String) -> Result<String> {
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
        pub fn new(s: HashMap<String, String>) -> Self {
            let mut default_file_read = Self::default();
            default_file_read.0.extend(s);
            default_file_read
        }
    }

    #[async_trait::async_trait]
    impl FileReadService for TestFileReadService {
        async fn read(&self, path: String) -> Result<String> {
            self.0.get(&path).cloned().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found: {}", path),
                )
                .into()
            })
        }
    }
}
