use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

#[async_trait::async_trait]
pub trait ToolCallService {
    type Input: DeserializeOwned;
    type Output: Serialize;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String>;
}

pub struct JsonTool<T>(T);

impl<T> JsonTool<T> {
    pub fn new(tool: T) -> Self {
        Self(tool)
    }
}

#[async_trait::async_trait]
impl<T: ToolCallService + Sync> ToolCallService for JsonTool<T>
where
    T::Input: serde::de::DeserializeOwned + JsonSchema,
    T::Output: serde::Serialize + JsonSchema,
{
    type Input = Value;
    type Output = Value;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let input: T::Input = serde_json::from_value(input).map_err(|e| e.to_string())?;
        let output: T::Output = self.0.call(input).await?;
        Ok(serde_json::to_value(output).map_err(|e| e.to_string())?)
    }
}
