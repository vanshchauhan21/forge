mod app;
mod broadcast;
mod completion;
mod engine;
mod error;
mod log;
mod server;

use std::convert::Infallible;

use axum::response::sse::Event;
pub use error::*;
use futures::Stream;
pub use server::Server;

type EventStream = Box<dyn Stream<Item = std::result::Result<Event, Infallible>> + Send + Unpin>;
