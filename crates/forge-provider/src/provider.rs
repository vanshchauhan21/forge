use super::error::Result;
use futures::stream::Stream;
use tracing::info;

#[async_trait::async_trait]
pub(crate) trait InnerProvider {
    fn name(&self) -> &'static str;
    async fn prompt(&self, input: String)
        -> Result<Box<dyn Stream<Item = Result<String>> + Unpin>>;
    async fn test(&self) -> Result<bool>;

    async fn models(&self) -> Result<Vec<String>>;
}

pub struct Provider {
    provider: Box<dyn InnerProvider>,
}

impl Provider {
    pub async fn test(&self) -> Result<bool> {
        let status = self.provider.test().await?;

        info!(
            "Connection Successfully established with {}",
            self.provider.name()
        );

        Ok(status)
    }

    pub async fn prompt(
        &self,
        input: String,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Unpin>> {
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
