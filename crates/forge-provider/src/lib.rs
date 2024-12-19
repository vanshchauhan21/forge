pub mod error;
mod open_ai;
mod open_router;
mod provider;

pub use provider::*;

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;
