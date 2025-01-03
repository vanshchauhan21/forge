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
    use super::*;

    pub struct TestFileReadService(String);

    impl TestFileReadService {
        pub fn new(s: impl ToString) -> Self {
            Self(s.to_string())
        }
    }

    #[async_trait::async_trait]
    impl FileReadService for TestFileReadService {
        async fn read(&self, _: String) -> Result<String> {
            Ok(self.0.clone())
        }
    }
}
