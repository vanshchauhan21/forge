use serde::de::DeserializeOwned;

#[async_trait::async_trait]
pub trait ToolCallService {
    type Input: DeserializeOwned;

    async fn call(&self, input: Self::Input) -> Result<String, String>;
}
