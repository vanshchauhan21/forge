use mcp_rs::{
    error::McpError,
    protocol::{JsonRpcRequest, JsonRpcResponse},
};

pub struct DynTool<T>(T);

#[async_trait::async_trait]
impl<T: Tool + Sync> Tool for DynTool<T>
where
    T::Input: TryFrom<JsonRpcRequest, Error = McpError>,
    T::Output: TryInto<JsonRpcResponse, Error = McpError>,
{
    type Input = JsonRpcRequest;
    type Output = JsonRpcResponse;

    async fn tools_call(&self, input: Self::Input) -> Result<Self::Output, McpError> {
        let input: T::Input = input.try_into()?;
        let output: JsonRpcResponse = self.0.tools_call(input).await?.try_into()?;
        Ok(output)
    }

    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn tools_list(&self) -> Vec<&'static str> {
        self.0.tools_list()
    }
}

#[async_trait::async_trait]
pub trait Tool {
    type Input;
    type Output;

    fn name(&self) -> &'static str;

    async fn tools_call(&self, input: Self::Input) -> Result<Self::Output, McpError>;

    fn tools_list(&self) -> Vec<&'static str>;

    fn into_dyn(self) -> DynTool<Self>
    where
        Self: Sized,
    {
        DynTool(self)
    }
}
