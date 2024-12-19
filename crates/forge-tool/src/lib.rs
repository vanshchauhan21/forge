mod tool;

pub use mcp_rs::protocol::{JsonRpcRequest, JsonRpcResponse};
pub use tool::*;

pub mod error {
    pub type Error = mcp_rs::error::McpError;
}
