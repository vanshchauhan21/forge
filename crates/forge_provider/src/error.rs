use derive_more::derive::Display;
use thiserror::Error;

#[derive(Debug, Display, derive_more::From, Error)]
pub enum Error {
    EmptyContent,
    #[from(ignore)]
    #[display("Upstream: {}", 1)]
    Upstream {
        code: u32,
        message: String,
    },
    SerdeJson(serde_json::Error),
    ToolCallMissingName,
}
