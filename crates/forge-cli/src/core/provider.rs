use super::error::Result;
use futures::stream::Stream;
use tracing::info;

pub trait InnerProvider {
    fn name(&self) -> &'static str;
    async fn prompt(&self, input: String)
        -> Result<Box<dyn Stream<Item = Result<String>> + Unpin>>;
    async fn test(&self) -> Result<bool>;
}

pub struct Provider<Provider> {
    provider: Provider,
}

impl<P: InnerProvider> Provider<P> {
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

    pub fn new(provider: P) -> Self {
        Self { provider }
    }
}
