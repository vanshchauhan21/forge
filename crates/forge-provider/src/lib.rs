pub mod error;
mod open_ai;
mod open_router;
mod provider;

// TODO: only expose Request & Response
pub use open_router::*;
pub use provider::*;

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;
