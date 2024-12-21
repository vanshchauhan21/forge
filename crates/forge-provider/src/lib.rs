pub mod error;
pub mod model;
#[allow(unused)]
mod open_ai;
mod open_router;
mod provider;

// TODO: only expose Request & Response
pub use provider::*;

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;
