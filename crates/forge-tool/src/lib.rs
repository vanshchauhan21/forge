use model::{CallToolRequest, CallToolResponse, ToolsListResponse};

mod fs;
mod model;
mod think;
pub use fs::FS;
pub use think::Think;

#[async_trait::async_trait]
pub trait Tool {
    fn name(&self) -> &'static str;
    async fn tools_call(&self, input: CallToolRequest) -> Result<CallToolResponse, String>;
    fn tools_list(&self) -> Result<ToolsListResponse, String>;
}
