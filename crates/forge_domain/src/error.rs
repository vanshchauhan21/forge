use std::pin::Pin;

use derive_more::derive::From;
use thiserror::Error;

#[derive(From, Debug, Error)]
pub enum Error {
    #[error("Tool name was not provided")]
    ToolCallMissingName,

    #[error("Serde Error: {0}")]
    Serde(serde_json::Error),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(uuid::Error),

    #[error("Invalid user command: {0}")]
    InvalidUserCommand(String),

    #[error("Template rendering error: {0}")]
    Template(handlebars::RenderError),
}

pub type Result<A> = std::result::Result<A, Error>;
pub type BoxStream<A, E> =
    Pin<Box<dyn tokio_stream::Stream<Item = std::result::Result<A, E>> + Send>>;

pub type ResultStream<A, E> = std::result::Result<BoxStream<A, E>, E>;
