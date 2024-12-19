pub mod error;
mod open_ai;
mod open_router;
mod provider;
mod request;
mod response;

// TODO: only expose Request & Response
pub use provider::*;
pub use request::*;
pub use response::*;

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;
