use std::sync::Arc;

use derive_more::derive::Display;
use serde_json::Value;

#[derive(Debug, Display, derive_more::From)]
pub enum Error {
    // Custom display message for provider error
    EmptyContent,
    Upstream(Value),
    Reqwest(#[from] reqwest::Error),
    SerdeJson(#[from] serde_json::Error),
    EventSource(#[from] reqwest_eventsource::Error),
    ToolCallMissingName,
    Arc(Arc<Error>),
}

pub type Result<T> = std::result::Result<T, Error>;
