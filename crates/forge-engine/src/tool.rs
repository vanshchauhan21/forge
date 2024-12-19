use serde_json::Value;

use crate::error::Result;
pub struct SerdeTool<T>(pub T);

#[async_trait::async_trait]
impl<T: Tool + Sync> Tool for SerdeTool<T>
where
    T::Input: serde::de::DeserializeOwned,
    T::Output: serde::Serialize,
{
    type Input = Value;
    type Output = Value;

    async fn run(&self, input: Value) -> Result<Value> {
        let input = serde_json::from_value(input)?;
        let output = self.0.run(input).await?;
        Ok(serde_json::to_value(output)?)
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }
}

#[async_trait::async_trait]
pub trait Tool {
    type Input;
    type Output;
    async fn run(&self, input: Self::Input) -> Result<Self::Output>;
    fn name(&self) -> &'static str;
}
