use std::pin::Pin;

use derive_more::derive::{Display, From};

#[derive(From, Debug, Display)]
pub enum Error {
    ToolUseMissingName,
    Serde(serde_json::Error),
}

pub type Result<A> = std::result::Result<A, Error>;

pub type UStream<A> = Pin<Box<dyn tokio_stream::Stream<Item = A> + Send>>;
pub type BoxStream<A, E> = UStream<std::result::Result<A, E>>;
pub type ResultStream<A, E> = std::result::Result<BoxStream<A, E>, E>;
