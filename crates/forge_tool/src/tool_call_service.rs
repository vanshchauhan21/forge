use serde::de::DeserializeOwned;
use serde::Serialize;

#[async_trait::async_trait]
pub trait ToolCallService {
    type Input: DeserializeOwned;
    type Output: Serialize;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String>;
}
