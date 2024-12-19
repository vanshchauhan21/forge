use crate::Stream;

use super::error::Result;

#[async_trait::async_trait]
pub(crate) trait InnerProvider {
    fn name(&self) -> &'static str;
    async fn prompt(&self, input: String) -> Result<Stream<Result<String>>>;

    async fn models(&self) -> Result<Vec<String>>;
}

pub struct Provider {
    provider: Box<dyn InnerProvider>,
}

impl Provider {
    pub async fn prompt(&self, input: String) -> Result<Stream<Result<String>>> {
        self.provider.prompt(input).await
    }

    pub async fn models(&self) -> Result<Vec<String>> {
        self.provider.models().await
    }

    pub(crate) fn new(provider: impl InnerProvider + 'static) -> Self {
        Self {
            provider: Box::new(provider),
        }
    }
}
